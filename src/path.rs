use std::collections::HashMap;

use itertools::Itertools;
use opencv::core::{bitwise_or, in_range, Mat, Rect, Vector};
use opencv::imgproc::{cvt_color, COLOR_BGR2HSV};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoWriter};
use toml::{Table, Value};

use crate::motor::Drivable;
use crate::remote::CarControl;

/// Angle between -90 (left) and 90 (right)
pub type Angle = f64;

/// Returns absolute distance between two points.
pub fn point_dist(first: &(f32, f32), second: &(f32, f32)) -> f32 {
    f32::sqrt((first.0 - second.0).powf(2.0) + (first.1 - second.1).powf(2.0))
}

pub fn img_index_to_coord(height: &i32, index: &i32) -> (i32, i32) {
    (index % height, index / height)
}

pub struct Frame<'a> {
    /// Mat of the left lines.
    left: &'a Mat,
    /// Mat of the right lines.
    right: &'a Mat,
    /// Mat of the obstacles (boxes/cars).
    obstacles: &'a Mat,
    /// Size of the frame.
    size: (i32, i32),
}

impl<'a> Frame<'a> {
    pub fn reference_point(&self) -> (i32, i32) {
        (self.size.0 / 2, self.size.1)
    }
}

/// Models the HSV thresholds for object detection.
pub struct ColorThresholds {
    pub left_lower: Vector<u8>,
    pub left_upper: Vector<u8>,
    pub right_lower: Vector<u8>,
    pub right_upper: Vector<u8>,
    pub box_lower: Vector<u8>,
    pub box_upper: Vector<u8>,
    pub car_lower: Vector<u8>,
    pub car_upper: Vector<u8>,
}

impl ColorThresholds {
    pub fn from_toml(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap();
        let table = content.parse::<Table>().unwrap();
        let mut vals = HashMap::new();
        for key in ["left", "right", "box", "car"] {
            vals.insert(key.to_owned(), Self::parse_threshold(&table, key));
        }

        let left = vals.get("left").unwrap();
        let right = vals.get("right").unwrap();
        let boxes = vals.get("box").unwrap();
        let car = vals.get("car").unwrap();
        return Self {
            left_lower: Vector::from(left.0.clone()),
            left_upper: Vector::from(left.1.clone()),
            right_lower: Vector::from(right.0.clone()),
            right_upper: Vector::from(right.1.clone()),
            box_lower: Vector::from(boxes.0.clone()),
            box_upper: Vector::from(boxes.1.clone()),
            car_lower: Vector::from(car.0.clone()),
            car_upper: Vector::from(car.1.clone()),
        };
    }

    fn parse_threshold(table: &Table, key: &str) -> (Vec<u8>, Vec<u8>) {
        let lower: Vec<u8> = match &table[&format!("{key}_lower")] {
            Value::Array(vals) => {
                let mut _lower = vec![];
                for val in vals {
                    match val {
                        Value::Integer(int) => {
                            _lower.push(*int as u8);
                        }
                        _ => panic!("Members of {key}_lower must be ints."),
                    }
                }
                _lower
            }
            _ => panic!("{key}_lower must be array."),
        };

        let upper: Vec<u8> = match &table[&format!("{key}_upper")] {
            Value::Array(vals) => {
                let mut _upper = vec![];
                for val in vals {
                    match val {
                        Value::Integer(int) => {
                            _upper.push(*int as u8);
                        }
                        _ => panic!("Members of {key}_upper must be ints."),
                    }
                }
                _upper
            }
            _ => panic!("{key}_upper must be array."),
        };

        (lower, upper)
    }
}

pub struct Pathfinder {
    pub angle: Angle,
    pub roi: Rect,
    pub car: CarControl,
    pub thresholds: ColorThresholds,
    debug_out: Option<VideoWriter>,
}

impl Pathfinder {
    pub fn new(
        car: CarControl,
        thresholds: ColorThresholds,
        debug_out: Option<VideoWriter>,
    ) -> Self {
        Pathfinder {
            angle: 0.0,
            roi: Rect {
                x: 0,
                y: 100,
                width: 640,
                height: 380,
            },
            car,
            thresholds,
            debug_out,
        }
    }

    /// Drives at angle determined by data read from cap.
    pub fn drive(&mut self, mut cap: VideoCapture) {
        while !self.car.is_enabled() {}
        let mut bgr_img = Mat::default();
        while let Ok(true) = cap.read(&mut bgr_img) {
            let angle = self.consider_frame(&bgr_img);
            println!("Angle: {angle}");
            self.car.angle(angle, 75);

            if !self.car.is_enabled() {
                if self.debug_out.is_some() {
                    self.debug_out.as_mut().unwrap().release().unwrap();
                }
                break;
            }
        }
    }

    /// Chooses an angle to drive at from the lines in the frame.
    /// Returns the angle most commonly suggested by the last 5 frames.
    pub fn consider_frame(&mut self, frame: &Mat) -> Angle {
        let mut hsv = Mat::default();
        cvt_color(&frame, &mut hsv, COLOR_BGR2HSV, 0).expect("Failed to convert img to HSV");
        let hsv_roi = Mat::roi(&hsv, self.roi).expect("Failed to slice region of HSV img.");

        let (mut left_mask, mut right_mask) = (Mat::default(), Mat::default());
        self.left_mask(&hsv_roi, &mut left_mask);
        self.right_mask(&hsv_roi, &mut right_mask);
        assert_eq!(left_mask.cols(), right_mask.cols());
        assert_eq!(left_mask.rows(), right_mask.rows());
        let mut obstacle_mask = Mat::default();
        self.obstacle_mask(&hsv_roi, &mut obstacle_mask);
        let frame = Frame {
            left: &left_mask,
            right: &right_mask,
            obstacles: &obstacle_mask,
            size: (left_mask.cols(), left_mask.rows()),
        };

        if self.debug_out.is_some() {
            let mut line_mask = Mat::default();
            bitwise_or(&left_mask, &right_mask, &mut line_mask, &Mat::default()).unwrap();
            // let mut debug_frame = Mat::default();
            // bitwise_or(
            //     &line_mask,
            //     &obstacle_mask,
            //     &mut debug_frame,
            //     &Mat::default(),
            // )
            // .unwrap();
            self.debug_out
                .as_mut()
                .unwrap()
                .write(&line_mask)
                .unwrap();
        }

        choose_angle(&frame)
    }

    pub fn left_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(
            src,
            &self.thresholds.left_lower,
            &self.thresholds.left_upper,
            dst,
        )
        .unwrap();
    }

    pub fn right_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(
            src,
            &self.thresholds.right_lower,
            &self.thresholds.right_upper,
            dst,
        )
        .unwrap();
    }

    pub fn obstacle_mask(&self, src: &Mat, dst: &mut Mat) {
        let mut box_mask = Mat::default();
        in_range(
            src,
            &self.thresholds.box_lower,
            &self.thresholds.box_upper,
            &mut box_mask,
        )
        .unwrap();
        let mut car_mask = Mat::default();
        in_range(
            src,
            &self.thresholds.car_lower,
            &self.thresholds.car_upper,
            &mut car_mask,
        )
        .unwrap();

        bitwise_or(&car_mask, &box_mask, dst, &Mat::default()).unwrap()
    }
}

pub fn choose_angle(frame: &Frame) -> Angle {
    let (mut best_angle, mut max_dist): (Angle, u32) = (0.0, 0);
    for angle in (0..900).step_by(5).interleave((-900..0).step_by(5).rev()) {
        let angle = angle as f64 / 10.0;
        match direction_from_ray(frame, &angle) {
            None => return angle,
            Some(dist) => {
                if dist > max_dist {
                    max_dist = dist;
                    best_angle = angle;
                }
            }
        }
    }
    best_angle
}

/// Casts a ray from the bottom centre at the given angle.
/// Returns a direction to turn based on what the ray hits.
pub fn direction_from_ray(frame: &Frame, angle: &Angle) -> Option<u32> {
    for point in cast_ray(&frame.size.0, &frame.size.1, angle) {
        let origin = (
            frame.reference_point().0 as f32,
            frame.reference_point().1 as f32,
        );
        let blocked = {
            if let Ok(255) = frame.left.at::<u8>(point) {
                true
            } else if let Ok(255) = frame.right.at::<u8>(point) {
                true
            } else if let Ok(255) = frame.obstacles.at::<u8>(point) {
                true
            } else {
                false
            }
        };
        if blocked {
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            return Some(point_dist(&origin, &coords) as u32);
        }
    }
    None
}

/// Casts a ray through the space of a given size, at a given angle
/// from the bottom centre. Returns the indices of all elements
/// on the ray.
pub fn cast_ray(width: &i32, height: &i32, angle: &Angle) -> Vec<i32> {
    let tan_angle = angle.to_radians().tan();
    let center = width / 2;
    let mut indices = Vec::new();
    for row in (0..*height).rev() {
        let offset = ((height - row) as f64) * tan_angle;
        if offset.abs() > center as f64 {
            break;
        }
        let index = (row * width) - center + offset as i32;
        indices.push(index);
    }
    indices
}

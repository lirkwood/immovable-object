use opencv::core::{in_range, Mat, Rect, Vector};
use opencv::imgproc::{cvt_color, COLOR_BGR2HSV};
use opencv::prelude::*;
use itertools::Itertools;
use opencv::videoio::VideoCapture;

use crate::motor::{Car, Drivable};
use crate::remote::CarControl;

/// Angle between -90 (left) and 90 (right)
pub type Angle = f32;

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
    /// Size of the frame.
    size: (i32, i32),
}

impl<'a> Frame<'a> {
    pub fn reference_point(&self) -> (i32, i32) {
        (self.size.0 / 2, self.size.1)
    }
}

/// Models a left or right boundary line.
/// Content is distance from bottom centre.
pub enum Line {
    Left(u32),
    Right(u32),
    Straight,
}

pub struct Pathfinder {
    pub angle: Angle,
    pub roi: Rect,
    pub car: CarControl,
    left_lower_hsv: Vector<u8>,
    left_upper_hsv: Vector<u8>,
    right_lower_hsv: Vector<u8>,
    right_upper_hsv: Vector<u8>,
}

impl Pathfinder {
    pub fn new(car: CarControl) -> Self {
        Pathfinder {
            angle: 0.0,
            roi: Rect {
                x: 0,
                y: 100,
                width: 640,
                height: 380,
            },
            car,
            left_lower_hsv: Vector::from(vec![23, 40, 40]),
            left_upper_hsv: Vector::from(vec![37, 255, 255]),
            right_lower_hsv: Vector::from(vec![95, 50, 50]),
            right_upper_hsv: Vector::from(vec![145, 255, 255]),
        }
    }

    /// Drives at angle determined by data read from cap.
    pub fn drive(&mut self, mut cap: VideoCapture) {
        let mut bgr_img = Mat::default();
        loop {
            match cap.read(&mut bgr_img) {
                Ok(true) => {}
                _ => break,
            }

            let angle = self.consider_frame(&mut bgr_img);
            self.car.angle(angle, 75);
        }
    }

    /// Chooses an angle to drive at from the lines in the frame.
    /// Returns the angle most commonly suggested by the last 5 frames.
    pub fn consider_frame(&mut self, frame: &Mat) -> Angle {
        let mut hsv = Mat::default();
        cvt_color(&frame, &mut hsv, COLOR_BGR2HSV, 0).expect("Failed to convert img to HSV");
        let hsv_roi =
            Mat::roi(&hsv, self.roi).expect("Failed to slice region of HSV img.");

        let (mut left_mask, mut right_mask) = (Mat::default(), Mat::default());
        self.left_mask(&hsv_roi, &mut left_mask);
        self.right_mask(&hsv_roi, &mut right_mask);
        assert_eq!(left_mask.cols(), right_mask.cols());
        assert_eq!(left_mask.rows(), right_mask.rows());
        let frame = Frame {
            left: &left_mask,
            right: &right_mask,
            size: (left_mask.cols(), left_mask.rows())
        };

        choose_angle(&frame)
    }

    pub fn left_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(
            src,
            &self.left_lower_hsv,
            &self.left_upper_hsv,
            dst,
        ).unwrap();
    }

    pub fn right_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(
            src,
            &self.right_lower_hsv,
            &self.right_upper_hsv,
            dst,
        ).unwrap();
    }
}

pub fn choose_angle(frame: &Frame) -> Angle {
    let (mut best_angle, mut max_dist): (Angle, u32) = (0.0, 0);
    for angle in (0..900).step_by(5).interleave(( -900..0 ).step_by(5).rev()) {
        let angle = angle as f32 / 10.0;
        match direction_from_ray(frame, &angle) {
            Line::Straight => return angle,
            Line::Left(dist) => {
                if dist > max_dist {
                    max_dist = dist;
                    best_angle = angle;
                }
            }
            Line::Right(dist) => {
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
pub fn direction_from_ray(frame: &Frame, angle: &Angle) -> Line {
    let ray = cast_ray(&frame.size.0, &frame.size.1, angle);
    for point in ray {
        let origin = (frame.reference_point().0 as f32, frame.reference_point().1 as f32);
        if let Ok(255) = frame.left.at::<u8>(point) {
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            return Line::Right(point_dist(&origin, &coords) as u32);
        } else if let Ok(255) = frame.right.at::<u8>(point) {
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            return Line::Left(point_dist(&origin, &coords) as u32);
        }
    }
    Line::Straight
}

/// Casts a ray through the space of a given size, at a given angle
/// from the bottom centre. Returns the indices of all elements
/// on the ray.
pub fn cast_ray(width: &i32, height: &i32, angle: &Angle) -> Vec<i32> {
    let tan_angle = angle.to_radians().tan();
    let center = width / 2;
    let mut indices = Vec::new();
    for row in (0..*height).rev() {
        let offset = ((height - row) as f32) * tan_angle;
        if offset.abs() > center as f32 {
            break;
        }
        let index = (row * width) - center + offset as i32;
        indices.push(index);
    }
    indices
}

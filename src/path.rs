use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Range;

use opencv::core::{bitwise_or, in_range, Mat, Rect, VecN, Vector};
use opencv::imgproc::{cvt_color, COLOR_BGR2HSV, COLOR_GRAY2BGR};
use opencv::prelude::*;
use opencv::videoio::{VideoCapture, VideoWriter};
use toml::{Table, Value};

use crate::motor::Drivable;
use crate::remote::CarControl;
use crate::tests::draw_ray;

/// Angle between -90 (left) and 90 (right)
pub type Angle = f64;

/// Returns absolute distance between two points.
pub fn point_dist(first: &(f32, f32), second: &(f32, f32)) -> f32 {
    f32::sqrt((first.0 - second.0).powf(2.0) + (first.1 - second.1).powf(2.0))
}

/// Converts the index of a point in an image to a coordinate (x, y)
pub fn img_index_to_coord(img_width: &i32, index: &i32) -> (i32, i32) {
    (index % img_width, index / img_width)
}

/// Models a frame of the track by its masks.
pub struct Frame<'a> {
    /// Mat of the left lines.
    left: &'a Mat,
    /// Mat of the right lines.
    right: &'a Mat,
    /// Mat of the obstacles (boxes/cars).
    obstacles: &'a Mat,
    /// Mat of the finish line.
    finish: &'a Mat,
    /// Size of the frame.
    size: (i32, i32),
}

impl<'a> Frame<'a> {
    pub fn reference_point(&self) -> (i32, i32) {
        (self.size.0 / 2, self.size.1)
    }
}

/// Models the HSV thresholds for object detection.
pub struct DrivableConfig {
    pub left_lower: Vector<u8>,
    pub left_upper: Vector<u8>,
    pub right_lower: Vector<u8>,
    pub right_upper: Vector<u8>,
    pub box_lower: Vector<u8>,
    pub box_upper: Vector<u8>,
    pub car_lower: Vector<u8>,
    pub car_upper: Vector<u8>,
    pub finish_lower: Vector<u8>,
    pub finish_upper: Vector<u8>,
    pub p_gain: f64,
    pub i_gain: f64,
    pub i_max: f64,
    pub speed: f64,
}

impl DrivableConfig {
    pub fn from_toml(path: &str) -> Self {
        let content = std::fs::read_to_string(path).unwrap();
        let table = content.parse::<Table>().unwrap();
        let mut vals = HashMap::new();
        for key in ["left", "right", "box", "car", "finish"] {
            vals.insert(key.to_owned(), Self::parse_threshold(&table, key));
        }

        let left = vals.get("left").unwrap();
        let right = vals.get("right").unwrap();
        let boxes = vals.get("box").unwrap();
        let car = vals.get("car").unwrap();
        let finish = vals.get("finish").unwrap();
        Self {
            left_lower: Vector::from(left.0.clone()),
            left_upper: Vector::from(left.1.clone()),
            right_lower: Vector::from(right.0.clone()),
            right_upper: Vector::from(right.1.clone()),
            box_lower: Vector::from(boxes.0.clone()),
            box_upper: Vector::from(boxes.1.clone()),
            car_lower: Vector::from(car.0.clone()),
            car_upper: Vector::from(car.1.clone()),
            finish_upper: Vector::from(finish.0.clone()),
            finish_lower: Vector::from(finish.1.clone()),
            p_gain: Self::parse_float(&table, "p_gain", Some(0.0..1.0)),
            i_gain: Self::parse_float(&table, "i_gain", Some(0.0..1.0)),
            i_max: Self::parse_float(&table, "i_max", None),
            speed: Self::parse_float(&table, "speed", Some(0.0..1.0)),
        }
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

    fn parse_float(table: &Table, key: &str, range: Option<Range<f64>>) -> f64 {
        match table[key] {
            Value::Float(val) => {
                if let Some(_range) = range {
                    if _range.contains(&val) {
                        return val;
                    } else {
                        panic!("Value {key} must be in range {_range:?}");
                    }
                }
                return val;
            }
            _ => panic!("Expected float for key {key}"),
        }
    }
}

/// Reads a video stream and tells a car which way to turn.
pub struct Pathfinder<T: Drivable> {
    /// Current driving angle.
    pub angle: Angle,
    /// Region of input to consider.
    pub roi: Rect,
    /// Car to drive.
    pub car: CarControl<T>,
    /// Thresholds to use to choose driving angle.
    pub config: DrivableConfig,
    /// Debug video output.
    debug_out: Option<VideoWriter>,
    /// Integral for PID controller.
    angle_integral: f64,
}

impl<T: Drivable + Send> Pathfinder<T> {
    pub fn new(car: CarControl<T>, config: DrivableConfig, debug_out: Option<VideoWriter>) -> Self {
        Pathfinder {
            angle: 0.0,
            roi: Rect {
                x: 0,
                y: 230,
                width: 640,
                height: 250,
            },
            car,
            config,
            debug_out,
            angle_integral: 0.0,
        }
    }

    /// Drives at angle determined by data read from cap.
    pub fn drive(&mut self, mut cap: VideoCapture) {
        while !self.car.is_enabled() {}
        let mut bgr_img = Mat::default();
        while let Ok(true) = cap.read(&mut bgr_img) {
            let angle = self.consider_frame(&bgr_img);
            let speed = self.config.speed * 100.0;
            println!("Angle: {angle}, Speed: {}", speed);
            self.car.angle(angle, speed as isize);

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

        let mut finish_mask = Mat::default();
        self.finish_mask(&hsv_roi, &mut finish_mask);

        let frame = Frame {
            left: &left_mask,
            right: &right_mask,
            obstacles: &obstacle_mask,
            finish: &finish_mask,
            size: (left_mask.cols(), left_mask.rows()),
        };

        let angle = self.smart_choose_angle(&frame);
        let ctrl_angle = self.pid_consider_angle(angle);

        // DEBUG
        if self.debug_out.is_some() {
            let mut line_mask = Mat::default();
            bitwise_or(&left_mask, &right_mask, &mut line_mask, &Mat::default()).unwrap();
            let mut bgr_lines = Mat::default();
            cvt_color(&line_mask, &mut bgr_lines, COLOR_GRAY2BGR, 0).unwrap();

            draw_ray(&mut bgr_lines, &angle, VecN::new(0.0, 0.0, 255.0, 255.0));
            self.debug_out.as_mut().unwrap().write(&bgr_lines).unwrap();
        }
        // DEBUG

        angle
    }

    /// Considers an angle as an input to the PID controller.
    /// Returns controlled value.
    fn pid_consider_angle(&mut self, mut angle: Angle) -> Angle {
        self.angle_integral += angle;
        if self.angle_integral > 200.0 {
            self.angle_integral = 200.0;
        } else if self.angle_integral < -200.0 {
            self.angle_integral = -200.0;
        }

        angle = (self.config.p_gain * angle) + (self.config.i_gain * self.angle_integral);
        if angle < -90.0 {
            angle = -90.0;
        } else if angle > 90.0 {
            angle = 90.0;
        }

        angle
    }

    /// Smarter choose_angle.
    pub fn smart_choose_angle(&mut self, frame: &Frame) -> Angle {
        let (mut best_angle, mut max_dist): (Angle, Option<u32>) = (0.0, None);
        let mut test_angles: VecDeque<f64> = VecDeque::from(vec![0.0]);
        let mut seen = HashSet::new();
        while let Some(angle) = test_angles.pop_front() {
            seen.insert(angle as i64);
            match ray_dist(frame, &angle) {
                None => return angle,
                Some(obj) => {
                    if let TrackObject::FinishLine(dist) = obj {
                        if dist < 10 {
                            self.car.disable();
                            break;
                        }
                        continue;
                    }

                    if obj.dist() < 150 {
                        if let Some(dist) = max_dist {
                            if obj.dist() > dist {
                                (best_angle, max_dist) = (angle, Some(obj.dist()));
                            }
                        } else {
                            (best_angle, max_dist) = (angle, Some(obj.dist()));
                        }
                    }

                    let new_angles = handle_track_obj(&seen, &angle, &obj);
                    if new_angles.len() == 0 {
                        break;
                    } else {
                        test_angles.extend(new_angles);
                    }
                }
            }
        }
        best_angle
    }

    pub fn left_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(src, &self.config.left_lower, &self.config.left_upper, dst).unwrap();
    }

    pub fn right_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(src, &self.config.right_lower, &self.config.right_upper, dst).unwrap();
    }

    pub fn obstacle_mask(&self, src: &Mat, dst: &mut Mat) {
        let mut box_mask = Mat::default();
        in_range(
            src,
            &self.config.box_lower,
            &self.config.box_upper,
            &mut box_mask,
        )
        .unwrap();
        let mut car_mask = Mat::default();
        in_range(
            src,
            &self.config.car_lower,
            &self.config.car_upper,
            &mut car_mask,
        )
        .unwrap();

        bitwise_or(&car_mask, &box_mask, dst, &Mat::default()).unwrap()
    }

    pub fn finish_mask(&self, src: &Mat, dst: &mut Mat) {
        in_range(
            src,
            &self.config.finish_lower,
            &self.config.finish_upper,
            dst,
        )
        .unwrap();
    }
}

/// Chooses angle to drive at from a frame.
// pub fn choose_angle(frame: &Frame) -> Angle {
//     let (mut best_angle, mut max_dist): (Angle, u32) = (0.0, 0);
//     for angle in (0..900).step_by(5).interleave((-900..0).step_by(5).rev()) {
//         let angle = angle as f64 / 10.0;
//         match ray_dist(frame, &angle) {
//             None => return angle,
//             Some(obstacle) => {
//                 let dist = match obstacle {
//                     TrackObject::LeftLine(dist)
//                     | TrackObject::RightLine(dist)
//                     | TrackObject::Obstacle(dist) => dist,
//                 };
//                 if dist > max_dist {
//                     max_dist = dist;
//                     best_angle = angle;
//                 }
//             }
//         }
//     }
//     best_angle
// }

pub enum TrackObject {
    LeftLine(u32),
    RightLine(u32),
    Obstacle(u32),
    FinishLine(u32),
}

impl TrackObject {
    pub fn dist(&self) -> u32 {
        match self {
            Self::LeftLine(dist)
            | Self::RightLine(dist)
            | Self::Obstacle(dist)
            | Self::FinishLine(dist) => *dist,
        }
    }
}

/// Returns next angles to check based on seen angles and seen track object.
pub fn handle_track_obj(seen: &HashSet<i64>, angle: &Angle, obj: &TrackObject) -> Vec<Angle> {
    let mut angles = vec![];
    match obj {
        TrackObject::FinishLine(_) => {}
        TrackObject::LeftLine(_) => {
            let mut new_angle = angle + 5.0;
            while seen.contains(&(new_angle as i64)) {
                new_angle += 5.0;
            }
            if new_angle <= 90.0 {
                angles.push(new_angle);
            }
        }
        TrackObject::RightLine(_) => {
            let mut new_angle = angle - 5.0;
            while seen.contains(&(new_angle as i64)) {
                new_angle -= 5.0;
            }
            if new_angle >= -90.0 {
                angles.push(new_angle);
            }
        }
        TrackObject::Obstacle(_) => {
            let mut right_angle = angle - 5.0;
            while seen.contains(&(right_angle as i64)) {
                right_angle -= 5.0;
            }
            if right_angle >= -90.0 {
                angles.push(right_angle);
            }

            let mut left_angle = angle + 5.0;
            while seen.contains(&(left_angle as i64)) {
                left_angle += 5.0;
            }
            if left_angle <= 90.0 {
                angles.push(right_angle);
            }
        }
    }
    angles
}

/// Returns all points around the center that are, at most, $dist points away.
/// Distance can be vertical horizontal or
fn surrounding_points(frame: &Mat, center: &i32, dist: i32) -> Vec<i32> {
    let mut points = vec![];
    points.extend(*center - dist..=*center + dist);
    for ring in 1..=dist {
        let vertical_dist = frame.cols() * ring;
        let (top_row, bot_row) = (*center - vertical_dist, *center + vertical_dist);
        points.extend(top_row - dist..=top_row + dist);
        points.extend(bot_row - dist..=bot_row + dist);
    }
    points
}

/// Checks all the points a given distance away.
/// Returns true if all of them are the target value.
fn inspect_point(mask: &Mat, center: &i32, dist: i32, target: u8) -> bool {
    for point in surrounding_points(mask, center, dist) {
        if mask.at::<u8>(point).is_ok_and(|val| *val != target) {
            return false;
        }
    }
    true
}

/// Casts a ray from the bottom centre at the given angle.
/// Returns the distance the ray travelled before hitting an obstacle.
pub fn ray_dist(frame: &Frame, angle: &Angle) -> Option<TrackObject> {
    for point in cast_ray(&frame.size.0, &frame.size.1, angle) {
        let mut blocked = None;

        if !inspect_point(frame.left, &point, 1, 0) {
            let origin = (
                frame.reference_point().0 as f32,
                frame.reference_point().1 as f32,
            );
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            blocked = Some(TrackObject::LeftLine(point_dist(&origin, &coords) as u32));
        } else if !inspect_point(frame.right, &point, 1, 0) {
            let origin = (
                frame.reference_point().0 as f32,
                frame.reference_point().1 as f32,
            );
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            blocked = Some(TrackObject::RightLine(point_dist(&origin, &coords) as u32));
        } else if !inspect_point(frame.obstacles, &point, 1, 0) {
            let origin = (
                frame.reference_point().0 as f32,
                frame.reference_point().1 as f32,
            );
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            blocked = Some(TrackObject::Obstacle(point_dist(&origin, &coords) as u32));
        } else if !inspect_point(frame.finish, &point, 1, 0) {
            let origin = (
                frame.reference_point().0 as f32,
                frame.reference_point().1 as f32,
            );
            let _coords = img_index_to_coord(&frame.size.0, &point);
            let coords = (_coords.0 as f32, _coords.1 as f32);
            blocked = Some(TrackObject::FinishLine(point_dist(&origin, &coords) as u32));
        }

        if blocked.is_some() {
            return blocked;
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

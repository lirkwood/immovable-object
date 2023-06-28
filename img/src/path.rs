use opencv::core::{in_range, Mat, Rect, Vector};
use opencv::imgproc::{cvt_color, COLOR_BGR2HSV};
use opencv::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// Angle between -90 (left) and 90 (right)
pub type Angle = f32;

/// Returns the angle from the line (first->second) to the line (first->straight up).
pub fn vertical_angle_to_point(first: &(f32, f32), second: &(f32, f32)) -> Angle {
    return ((first.0 - second.0) / point_dist(first, second))
        .asin()
        .to_degrees() as Angle;
}

/// Returns absolute distance between two points.
pub fn point_dist(first: &(f32, f32), second: &(f32, f32)) -> f32 {
    return f32::sqrt((first.0 - second.0).powf(2.0) + (first.1 - second.1).powf(2.0));
}

pub fn img_index_to_coord(height: &i32, index: &i32) -> (i32, i32) {
    return (index % height, index / height);
}

macro_rules! cast {
    ($target: ty, $($elem: expr), +) => {
        ($($elem as $target),+)
    };
}

pub enum Direction {
    Left,
    Right,
    Straight,
}

pub struct Pathfinder {
    pub angle: Angle,
    pub roi: Rect,
    angle_buf: VecDeque<Angle>,
    left_lower_hsv: Vector<u8>,
    left_upper_hsv: Vector<u8>,
    right_lower_hsv: Vector<u8>,
    right_upper_hsv: Vector<u8>,
}

impl Pathfinder {
    pub fn new() -> Self {
        return Pathfinder {
            angle: 0.0,
            roi: Rect {
                x: 0,
                y: 100,
                width: 640,
                height: 380,
            },
            angle_buf: VecDeque::new(),
            left_lower_hsv: Vector::from(vec![23, 40, 40]),
            left_upper_hsv: Vector::from(vec![37, 255, 255]),
            right_lower_hsv: Vector::from(vec![105, 60, 60]),
            right_upper_hsv: Vector::from(vec![135, 255, 255]),
        };
    }

    /// Chooses an angle to drive at from the lines in the frame.
    /// Returns the angle most commonly suggested by the last 5 frames.
    pub fn consider_frame(&mut self, frame: &Mat) -> Angle {
        let mut hsv = Mat::default();
        cvt_color(&frame, &mut hsv, COLOR_BGR2HSV, 0).expect("Failed to convert img to HSV");
        let mut hsv_roi =
            Mat::roi(&hsv, self.roi.clone()).expect("Failed to slice region of HSV img.");

        let (mut left_mask, mut right_mask) = (Mat::default(), Mat::default());
        self.left_mask(&hsv_roi, &mut left_mask);
        self.right_mask(&hsv_roi, &mut right_mask);

        return choose_angle(&left_mask, &right_mask, &200.0);
        // self.angle_buf
        //     .push_back(choose_angle(&left_mask, &right_mask, &200.0));
        // if self.angle_buf.len() > 3 {
        //     self.angle_buf.pop_front();
        // }

        // self.angle = tally_angles(&self.angle_buf);
        // return self.angle.clone();
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

/// Counts the angles in the buffer and selects the most common one.
fn tally_angles<'a>(buf: impl IntoIterator<Item = &'a Angle>) -> Angle {
    let mut angle_votes = HashMap::new();
    for angle in buf {
        *angle_votes.entry(angle.clone() as i32).or_default() += 1;
    }

    let (mut max_votes, mut max_angle) = (0, 0);
    for (angle, votes) in angle_votes {
        if votes > max_votes {
            (max_votes, max_angle) = (votes, angle)
        }
    }
    return max_angle as Angle;
}

pub fn choose_angle(left: &Mat, right: &Mat, max_dist: &f32) -> Angle {
    let mut seen = HashSet::new();
    let mut angle: Angle = 0.0;
    loop {
        match direction_from_ray(left, right, &angle, max_dist) {
            Direction::Straight => return angle,
            Direction::Left => {
                if angle <= -90.0 {
                    return -90.0;
                } else {
                    angle -= 2.5;
                }
            }
            Direction::Right => {
                if angle >= 90.0 {
                    return 90.0;
                } else {
                    angle += 2.5;
                }
            }
        }
        if seen.contains(&(angle as i32)) {
            return angle;
        } else {
            seen.insert(angle as i32);
        }
    }
}

/// Casts a ray from the bottom centre at the given angle.
/// Returns a direction to turn based on what the ray hits.
pub fn direction_from_ray(left: &Mat, right: &Mat, angle: &Angle, max_dist: &f32) -> Direction {
    assert_eq!(left.rows(), right.rows());
    assert_eq!(left.cols(), right.cols());
    let origin = ((left.cols() / 2) as f32, left.rows() as f32);
    let ray = cast_ray(&left.rows(), &left.cols(), angle);
    for point in ray {
        let y_coord = img_index_to_coord(&left.cols(), &point).1 as f32;
        let dist = point_dist(&origin, &(origin.0, y_coord));
        if &dist > max_dist {
            break;
        }
        if let Ok(255) = left.at::<u8>(point) {
            return Direction::Right;
        } else if let Ok(255) = right.at::<u8>(point) {
            return Direction::Left;
        }
    }
    return Direction::Straight;
}

/// Casts a ray through the space of a given size, at a given angle
/// from the bottom centre. Returns the indices of all elements
/// on the ray.
pub fn cast_ray(width: &i32, height: &i32, angle: &Angle) -> Vec<i32> {
    let tan_angle = angle.to_radians().tan();
    // println!("angle: {angle} tan: {tan_angle}");
    let center = width / 2;
    let mut indices = Vec::new();
    for row in (0..*height).rev() {
        let offset = ((height - row) as f32) * tan_angle;
        // println!("row {row} offset {offset}");
        if offset.abs() > center as f32 {
            break;
        }
        let index = (row * width) - center + offset as i32;
        indices.push(index);
    }
    return indices;
}

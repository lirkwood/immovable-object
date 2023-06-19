use opencv::core::Mat;
use opencv::prelude::*;
use std::collections::{HashMap, VecDeque};

/// Angle between -90 (left) and 90 (right)
pub type Angle = i32;

pub struct Polar {
    r: i32,
    theta: Angle
}

pub struct Pathfinder {
    frame_size: (u32, u32),
}

impl Pathfinder {
    /// Returns a new pathfinder with the default thresholds.
    pub fn new(size: (u32, u32)) -> Self {
        return Pathfinder { frame_size: size };
    }

    const NUM_SEGMENTS: u32 = 4;

    pub fn read_frame(&self, left_lines: &Mat, right_lines: &Mat) -> Vec<(f32, f32)> {
        let segment = (self.frame_size.1 / Self::NUM_SEGMENTS) as u32;

        let mut last_anchor = ((self.frame_size.0 / 2) as f32, self.frame_size.1 as f32);
        let mut segments = vec![last_anchor];
        let mut ballot = HashMap::new();

        for row_num in (0..self.frame_size.1).step_by(1) {
            let left_row = left_lines
                .row(row_num as i32)
                .expect(&format!("Failed to get left row {}", row_num));

            let right_row = right_lines
                .row(row_num as i32)
                .expect(&format!("Failed to get right row {}", row_num));

            let mut left_pts = row_line_cols(left_row);
            let mut right_pts = row_line_cols(right_row);

            // for every combo of left and right boundary, add vote
            // for angle rounded to nearest 10 to ballot. O(n^2) :(
            if left_pts.len() == 0 {
                left_pts.push(0);
            }
            if right_pts.len() == 0 {
                right_pts.push(self.frame_size.0 as u16);
            }
            for left_xpos in &left_pts {
                for right_xpos in &right_pts {
                    let centre = (right_xpos + left_xpos) / 2;
                    // let angle = centre_angle_to_point(last_anchor, (center, row_num as f32));
                    // let rounded_angle = (angle / 5) as i8 * 5;
                    if let None = ballot.get(&centre) {
                        ballot.insert(centre, 1);
                    } else {
                        ballot.insert(centre, ballot.get(&centre).unwrap() + 1);
                    }
                }
            }

            if row_num % segment == 0 {
                let mut winner = 0;
                let mut top_votes = 0;
                for (angle, votes) in &ballot {
                    if votes > &top_votes {
                        top_votes = *votes;
                        winner = *angle;
                    }
                }

                last_anchor = (
                    last_anchor.0 + ((winner / 45) as u32 * segment) as f32,
                    last_anchor.1 - segment as f32,
                );
                segments.push(last_anchor);
                // println!("Segment {}: {:?}", (row_num / segment) as u8, last_anchor);
                ballot.clear();
            }
        }
        return segments;
    }
}

pub fn centre_angle_to_point(first: (f32, f32), second: (f32, f32)) -> Angle {
    return (
        (first.0 - second.0) / point_dist(first, second)
    ).asin().to_degrees() as Angle;
}

/// Returns absolute distance between two points.
pub fn point_dist(first: (f32, f32), second: (f32, f32)) -> f32 {
    return f32::sqrt(
        (first.0 - second.0).powf(2.0) + (first.1 - second.1).powf(2.0)
    );
}

/// Number of columns that must be filled or empty
/// for it to be considered a line.
const HORIZONTAL_BUF_SIZE: usize = 5;

/// Gets the average column number of clusters of
/// 255 values from an array of 0|255 values.
pub fn row_line_cols(row: Mat) -> Vec<u16> {
    let mut indices = Vec::new();
    let mut buffer = VecDeque::from([false; HORIZONTAL_BUF_SIZE]);

    let mut cluster_start = None;

    let row_data = row
        .data_bytes()
        .expect("Failed to get data for cluster cols from row.");
    for col_num in 0..row.cols() {
        let value = row_data
            .get(col_num as usize)
            .expect(&format!("Row has no column number {}", col_num));

        if value != &0 {
            buffer.push_back(true);
        } else {
            buffer.push_back(false);
        }
        buffer.pop_front();

        // TODO optimise line below
        if buffer.iter().all(|e| e == &buffer[0]) {
            // All elem true
            if buffer[0] == true {
                cluster_start = match cluster_start {
                    Some(i) => Some(i),
                    None => Some(col_num),
                }
            // All elem false and cluster start exists
            } else if cluster_start.is_some() {
                indices.push(((col_num + cluster_start.unwrap()) / 2) as u16);
                cluster_start = None;
            }
        }
    }

    return indices;
}

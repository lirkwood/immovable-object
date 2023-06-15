use opencv::core::Mat;
use opencv::prelude::*;
use std::collections::VecDeque;

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

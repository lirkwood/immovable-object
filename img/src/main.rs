#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use opencv::{
    core::{in_range, Point, Rect, Scalar, Size, Vector},
    imgproc::{circle, cvt_color, COLOR_BGR2HSV, COLOR_HSV2BGR, LINE_8},
    prelude::*,
    videoio::{VideoCapture, VideoCaptureTrait, VideoWriter, VideoWriterTrait, CAP_ANY},
};

fn main() {
    let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
        .expect("Failed to read track video file.");
    let mut out = VideoWriter::new(
        "/home/linus/media/lines.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 480),
        true,
    )
    .expect("Failed to open lines video file for writing.");
    read(
        &mut cap,
        &mut out,
        Rect {
            x: 0,
            y: 100,
            width: 640,
            height: 380,
        },
    );
}

pub fn read(cap: &mut VideoCapture, out: &mut VideoWriter, roi: Rect) {
    let left_lower_hsv: Vector<u8> = Vector::from(vec![23, 40, 40]);
    let left_upper_hsv: Vector<u8> = Vector::from(vec![37, 255, 255]);

    let right_lower_hsv: Vector<u8> = Vector::from(vec![105, 40, 40]);
    let right_upper_hsv: Vector<u8> = Vector::from(vec![135, 255, 255]);

    let mut bgr_img = Mat::default();
    let mut hsv_img = Mat::default();
    let mut left_mask = Mat::default();
    let mut right_mask = Mat::default();

    let mut frame = 0;
    loop {
        frame += 1;
        match cap.read(&mut bgr_img) {
            Ok(true) => {}
            _ => break,
        }

        cvt_color(&mut bgr_img, &mut hsv_img, COLOR_BGR2HSV, 0)
            .expect("Failed to convert img to HSV");
        let mut hsv_roi =
            Mat::roi(&hsv_img, roi.clone()).expect("Failed to slice region of HSV img.");

        // Apply yellow/blue color threshold
        in_range(
            &mut hsv_roi,
            &left_lower_hsv,
            &left_upper_hsv,
            &mut left_mask,
        )
        .expect("Failed to apply left line colour threshold");
        in_range(
            &mut hsv_roi,
            &right_lower_hsv,
            &right_upper_hsv,
            &mut right_mask,
        )
        .expect("Failed to apply right line colour threshold");

        draw_clusters(&mut hsv_roi, &mut left_mask);
        draw_clusters(&mut hsv_roi, &mut right_mask);

        let mut bgr_img_final = Mat::default();
        cvt_color(&hsv_img, &mut bgr_img_final, COLOR_HSV2BGR, 0).unwrap();
        out.write(&bgr_img_final)
            .expect("Failed to write video frame.");
        println!("Wrote frame number {}", frame);
    }
    out.release().unwrap();
}

fn draw_clusters(img: &mut Mat, src: &mut Mat) {
    for row_num in (0..src.rows()).step_by(4) {
        let row = src
            .row(row_num)
            .expect(&format!("Left mask does have a row {}", row_num));

        for x_val in row_cluster_cols(row) {
            circle(
                img,
                Point {
                    x: x_val.into(),
                    y: row_num,
                },
                5,
                Scalar::new(240 as f64, 100 as f64, 100 as f64, 255 as f64),
                -1,
                LINE_8,
                0,
            )
            .expect("Failed to draw circle on image.");
        }
    }
}

const HORIZONTAL_BUF_SIZE: usize = 5;

/// Gets the average column number of clusters of
/// 255 values from an array of 0|255 values.
fn row_cluster_cols(row: Mat) -> Vec<u16> {
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

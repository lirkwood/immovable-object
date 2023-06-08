use opencv::{
    imgproc::{cvt_color, threshold, COLOR_BGR2HSV_FULL},
    prelude::Mat,
    videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY},
};
use std::env::args;

fn main() {
    let mut cap =
        VideoCapture::from_file(&args().nth(1).expect("Missing input file path."), CAP_ANY)
            .expect("Failed to read input file.");

    loop {
        let mut img = Mat::default();
        match cap.read(&mut img) {
            Ok(true) => {}
            _ => break,
        }

        let mut hsv = Mat::default();
        cvt_color(&mut img, &mut hsv, COLOR_BGR2HSV_FULL, 0).expect("Failed to convert img to HSV");

        let mut mask = Mat::default();
        threshold(hsv, mask, )
    }
}

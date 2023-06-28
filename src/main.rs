mod path;
#[allow(clippy::all)]
mod motor;
#[cfg(test)]
mod tests;

use opencv::{
    core::{bitwise_or, Point, Size, VecN},
    imgproc::{circle, cvt_color, COLOR_BGR2HSV, LINE_8, COLOR_GRAY2BGR},
    prelude::*,
    videoio::{VideoCapture, VideoCaptureTrait, VideoWriter, VideoWriterTrait, CAP_ANY},
};
use path::Pathfinder;

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

    let mut bgr_img = Mat::default();
    let mut pf = Pathfinder::new();
    let mut frame = 0;
    loop {
        frame += 1;
        match cap.read(&mut bgr_img) {
            Ok(true) => {}
            _ => break,
        }
        if frame > 1000 {
            break;
        }

        let mut hsv = Mat::default();
        cvt_color(&bgr_img, &mut hsv, COLOR_BGR2HSV, 0).unwrap();
        let mut left_mask = Mat::default();
        let mut right_mask = Mat::default();
        pf.left_mask(&hsv, &mut left_mask);
        pf.right_mask(&hsv, &mut right_mask);

        let mut combined_mask = Mat::default();
        bitwise_or(&left_mask, &right_mask, &mut combined_mask, &Mat::default()).unwrap();
        let mut bgr_final = Mat::default();
        cvt_color(&combined_mask, &mut bgr_final, COLOR_GRAY2BGR, 0).unwrap();
        let angle = pf.consider_frame(&bgr_img);
        draw_ray(&mut bgr_final, &angle, VecN([255.0, 0.0, 0.0, 255.0]));
        out.write(&bgr_final).unwrap();
        // draw_ray(&mut bgr_img, &angle, VecN([255.0, 0.0, 0.0, 255.0]));
        // out.write(&bgr_img).unwrap();
        println!("Frame {frame}");
    }
}

fn draw_ray(img: &mut Mat, angle: &path::Angle, color: VecN<f64, 4>) {
    for point in path::cast_ray(&img.cols(), &img.rows(), angle) {
        circle(
            img,
            Point::from(path::img_index_to_coord(&img.cols(), &point)),
            5,
            color,
            -1,
            LINE_8,
            0,
        )
        .unwrap();
    }
}

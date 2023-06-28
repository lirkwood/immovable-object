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

// pub fn read(cap: &mut VideoCapture, out: &mut VideoWriter, roi: Rect) {
//     let left_lower_hsv: Vector<u8> = Vector::from(vec![23, 40, 40]);
//     let left_upper_hsv: Vector<u8> = Vector::from(vec![37, 255, 255]);
//     let right_lower_hsv: Vector<u8> = Vector::from(vec![105, 60, 60]);
//     let right_upper_hsv: Vector<u8> = Vector::from(vec![135, 255, 255]);

//     let mut bgr_img = Mat::default();
//     let mut hsv_img = Mat::default();
//     let mut bgr_img_final = Mat::default();
//     let mut left_mask = Mat::default();
//     let mut right_mask = Mat::default();
//     let mut combined_mask = Mat::default();

//     let mut angle: path::Angle = 0.0;
//     let mut frame = 0;
//     loop {
//         frame += 1;
//         match cap.read(&mut bgr_img) {
//             Ok(true) => {}
//             _ => break,
//         }

//         cvt_color(&mut bgr_img, &mut hsv_img, COLOR_BGR2HSV, 0)
//             .expect("Failed to convert img to HSV");
//         let mut hsv_roi =
//             Mat::roi(&hsv_img, roi.clone()).expect("Failed to slice region of HSV img.");

//         // Apply yellow/blue color threshold
//         in_range(
//             &mut hsv_roi,
//             &left_lower_hsv,
//             &left_upper_hsv,
//             &mut left_mask,
//         )
//         .expect("Failed to apply left line colour threshold");
//         in_range(
//             &mut hsv_roi,
//             &right_lower_hsv,
//             &right_upper_hsv,
//             &mut right_mask,
//         )
//         .expect("Failed to apply right line colour threshold");

//         // cvt_color(&hsv_img, &mut bgr_img_final, COLOR_HSV2BGR, 0).unwrap();
//         bitwise_or(&left_mask, &right_mask, &mut combined_mask, &Mat::default()).unwrap();
//         cvt_color(&combined_mask, &mut bgr_img_final, COLOR_GRAY2RGB, 0).unwrap();

//         if (frame % 10) != 0 {
//             angle = path::choose_angle(&left_mask, &right_mask);
//         }
//         draw_ray(
//             &mut bgr_img_final,
//             &(-angle),
//             VecN::new(255.0, 0.0, 0.0, 255.0),
//         );

//         out.write(&bgr_img_final)
//             .expect("Failed to write video frame.");
//         println!("Wrote frame number {}", frame);
//     }
//     out.release().unwrap();
// }

fn draw_ray(img: &mut Mat, angle: &path::Angle, color: VecN<f64, 4>) {
    for point in path::cast_ray(&img.cols(), &img.rows(), &(angle)) {
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

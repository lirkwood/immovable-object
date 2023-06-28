
use crate::path::point_dist;



// #[test]
// pub fn test_row_cluster_indices() {
//     let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
//         .expect("Failed to read track video file.");
//     let mut out = VideoWriter::new(
//         "/home/linus/media/lines.mp4",
//         VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
//         30.0,
//         Size::new(640, 480),
//         true,
//     )
//     .expect("Failed to open lines video file for writing.");
//     read(
//         &mut cap,
//         &mut out,
//         Rect {
//             x: 0,
//             y: 100,
//             width: 640,
//             height: 380,
//         },
//     );
// }

#[test]
pub fn test_point_dist() {
    assert_eq!(point_dist(&(1.0, 1.0), &(2.0, 1.0)), 1.0);
    assert_eq!(point_dist(&(1.0, 1.0), &(1.0, 2.0)), 1.0);
    assert_eq!(point_dist(&(1.0, 1.0), &(2.0, 2.0)), f32::sqrt(2.0));
}

// #[test]
// pub fn test_get_combined_mask() {
//     let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
//         .expect("Failed to read track video file.");
//     let mut out = VideoWriter::new(
//         "/home/linus/media/lines.mp4",
//         VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
//         30.0,
//         Size::new(640, 380),
//         false,
//     )
//     .expect("Failed to open lines video file for writing.");

//     let left_lower_hsv: Vector<u8> = Vector::from(vec![23, 40, 40]);
//     let left_upper_hsv: Vector<u8> = Vector::from(vec![37, 255, 255]);
//     let right_lower_hsv: Vector<u8> = Vector::from(vec![60, 40, 40]);
//     let right_upper_hsv: Vector<u8> = Vector::from(vec![150, 255, 255]);

//     let mut bgr_img = Mat::default();
//     let mut hsv_img = Mat::default();

//     let mut left_mask = Mat::default();
//     let mut right_mask = Mat::default();
//     let mut combined_mask = Mat::default();

//     loop {
//         match cap.read(&mut bgr_img) {
//             Ok(true) => {}
//             _ => break,
//         }

//         cvt_color(&mut bgr_img, &mut hsv_img, COLOR_BGR2HSV, 0)
//             .expect("Failed to convert img to HSV");
//         let mut hsv_roi = Mat::roi(
//             &hsv_img,
//             Rect {
//                 x: 0,
//                 y: 100,
//                 width: 640,
//                 height: 380,
//             },
//         )
//         .expect("Failed to slice region of HSV img.");

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

//         bitwise_or(&left_mask, &right_mask, &mut combined_mask, &Mat::default()).unwrap();
//         out.write(&combined_mask).unwrap();
//     }
//     out.release().unwrap();
// }

// #[test]
// fn test_video_rw() {
//     let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
//         .expect("Failed to read track video file.");
//     let mut out = VideoWriter::new(
//         "/home/linus/media/lines.mp4",
//         VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
//         30.0,
//         Size::new(640, 480),
//         true,
//     ).expect("Failed to open lines video file for writing.");

//     let mut img = Mat::default();
//     loop {
//         match cap.read(&mut img) {
//             Ok(true) => {},
//             _ => break
//         }

//         out.write(&img).unwrap();
//     }
//     out.release().unwrap();
// }

// #[test]
// fn test_bgr2hsv_rw() {
//     let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
//         .expect("Failed to read track video file.");
//     let mut out = VideoWriter::new(
//         "/home/linus/media/lines.mp4",
//         VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
//         30.0,
//         Size::new(640, 480),
//         true,
//     ).expect("Failed to open lines video file for writing.");

//     let mut bgr_img = Mat::default();
//     let mut hsv_img = Mat::default();
//     loop {
//         match cap.read(&mut bgr_img) {
//             Ok(true) => {},
//             _ => break
//         }

//         cvt_color(&mut bgr_img, &mut hsv_img, COLOR_BGR2HSV, 0)
//             .expect("Failed to convert img to HSV");
//         out.write(&hsv_img).unwrap();
//     }
//     out.release().unwrap();
// }

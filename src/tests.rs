use crate::*;
use opencv::core::Size;
use opencv::videoio::CAP_ANY;

#[test]
pub fn test_row_cluster_indices() {
    let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
        .expect("Failed to read track video file.");
    let mut out = VideoWriter::new(
        "/home/linus/media/lines.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 480),
        true,
    ).expect("Failed to open lines video file for writing.");
    read(&mut cap, &mut out, Rect {
        x: 0, y: 100, width: 640, height: 380
    });
}

// #[test]
fn test_video_rw() {
    let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
        .expect("Failed to read track video file.");
    let mut out = VideoWriter::new(
        "/home/linus/media/lines.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 480),
        true,
    ).expect("Failed to open lines video file for writing.");

    let mut img = Mat::default();
    let mut frame = 0;
    loop {
        frame += 1;
        match cap.read(&mut img) {
            Ok(true) => {},
            _ => break
        }

        out.write(&img).unwrap();
    }
    out.release().unwrap();
}


// #[test]
fn test_bgr2hsv_rw() {
    let mut cap = VideoCapture::from_file("/home/linus/media/track.mp4", CAP_ANY)
        .expect("Failed to read track video file.");
    let mut out = VideoWriter::new(
        "/home/linus/media/lines.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 480),
        true,
    ).expect("Failed to open lines video file for writing.");

    let mut bgr_img = Mat::default();
    let mut hsv_img = Mat::default();
    let mut frame = 0;
    loop {
        frame += 1;
        match cap.read(&mut bgr_img) {
            Ok(true) => {},
            _ => break
        }


        cvt_color(&mut bgr_img, &mut hsv_img, COLOR_BGR2HSV, 0)
            .expect("Failed to convert img to HSV");
        out.write(&hsv_img).unwrap();
    }
    out.release().unwrap();
}

mod motor;
mod path;
mod remote;
#[cfg(test)]
mod tests;

use motor::Car;
use opencv::{
    core::{Size, Vector},
    videoio::{VideoCapture, VideoWriter, CAP_ANY},
};
use path::{ColorThresholds, Pathfinder};
use remote::CarControl;
use std::thread;

/// Prod main fn
// fn main() {
//     let car = CarControl::new(Car::default());
//     let clone = car.clone();
//     thread::spawn(|| remote::serve(clone));
//     Pathfinder::new(car, default_thresholds(), None).drive(VideoCapture::new(0, CAP_ANY).unwrap());
// }

/// Test main fn
fn main() {
    let car = CarControl::new(Car::default());
    let clone = car.clone();
    thread::spawn(|| remote::serve(clone));
    let debug_out = VideoWriter::new(
        "vision.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 380),
        false,
    )
    .unwrap();
    Pathfinder::new(
        car,
        ColorThresholds::from_toml("thresholds.toml"),
        Some(debug_out),
    )
    .drive(VideoCapture::new(0, CAP_ANY).unwrap());
}

fn default_thresholds() -> ColorThresholds {
    return ColorThresholds {
        left_lower: Vector::from(vec![23, 40, 40]),
        left_upper: Vector::from(vec![37, 255, 255]),
        right_lower: Vector::from(vec![95, 40, 40]),
        right_upper: Vector::from(vec![145, 255, 255]),
        box_lower: Vector::from(vec![0, 0, 0]),
        box_upper: Vector::from(vec![0, 0, 0]),
        car_lower: Vector::from(vec![0, 0, 0]),
        car_upper: Vector::from(vec![0, 0, 0]),
        // TODO add obstacle thresholds
    };
}

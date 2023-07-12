mod motor;
mod path;
mod remote;
mod tests;

use motor::Drivable;
use opencv::{
    core::{Size, Vector},
    videoio::{VideoWriter, VideoCapture, CAP_ANY, CAP_PROP_BUFFERSIZE},
    prelude::*
};
use std::thread;
use path::{DrivableConfig, Pathfinder};
use remote::CarControl;
use motor::Car;

use tests::DummyCar;

/// Prod main fn
fn main() {
    let car = CarControl::new(Car::default());
    let clone = car.clone();
    let debug_out = VideoWriter::new(
        "vision.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 250),
        true,
    )
    .unwrap();
    thread::spawn(|| remote::serve(clone));

    let mut cap = VideoCapture::new(0, CAP_ANY).unwrap();
    cap.set(CAP_PROP_BUFFERSIZE, 1.0).unwrap();
    Pathfinder::new(car, DrivableConfig::from_toml("thresholds.toml"), Some(debug_out)).drive(cap);
}

// fn main() {
//     let mut car = CarControl::new(DummyCar::new());
//     car.enable();
//     let debug_out = VideoWriter::new(
//         "vision.mp4",
//         VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
//         30.0,
//         Size::new(640, 380),
//         true,
//     )
//     .unwrap();

//     Pathfinder::new(
//         car,
//         DrivableConfig::from_toml("thresholds.toml"),
//         Some(debug_out),
//     )
//     .drive(tests::debug_in());
// }

// fn default_thresholds() -> DrivableConfig {
//     DrivableConfig {
//         left_lower: Vector::from(vec![23, 40, 40]),
//         left_upper: Vector::from(vec![37, 255, 255]),
//         right_lower: Vector::from(vec![95, 40, 40]),
//         right_upper: Vector::from(vec![145, 255, 255]),
//         box_lower: Vector::from(vec![0, 0, 0]),
//         box_upper: Vector::from(vec![0, 0, 0]),
//         car_lower: Vector::from(vec![0, 0, 0]),
//         car_upper: Vector::from(vec![0, 0, 0]),
//         // TODO add obstacle thresholds
//     }
// }

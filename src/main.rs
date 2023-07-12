mod motor;
mod path;
mod remote;
#[allow(dead_code)]
mod tests;

use opencv::{
    core::Size,
    videoio::{VideoWriter, VideoCapture, CAP_ANY, CAP_PROP_BUFFERSIZE},
    prelude::*
};
use std::thread;
use path::{DrivableConfig, Pathfinder};
use remote::CarControl;
use motor::Car;

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

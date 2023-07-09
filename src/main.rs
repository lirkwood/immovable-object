mod motor;
mod path;
mod remote;
#[cfg(test)]
mod tests;

use motor::Car;
use opencv::{
    core::Size,
    videoio::{VideoCapture, VideoWriter, CAP_ANY},
};
use path::Pathfinder;
use remote::CarControl;
use std::thread;

fn main() {
    let car = CarControl::new(Car::default());
    let clone = car.clone();
    thread::spawn(|| remote::serve(clone));
    Pathfinder::new(car, None).drive(VideoCapture::new(0, CAP_ANY).unwrap());
}

fn test() {
    let car = CarControl::new(Car::default());
    let clone = car.clone();
    thread::spawn(|| remote::serve(clone));
    let debug_out = VideoWriter::new(
        "vision.mp4",
        VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
        30.0,
        Size::new(640, 380),
        false,
    ).unwrap();
    Pathfinder::new(car, Some(debug_out)).drive(VideoCapture::new(0, CAP_ANY).unwrap());
}

mod motor;
mod path;
mod remote;
#[cfg(test)]
mod tests;

use motor::Car;
use opencv::{
    core::{Point, Size, VecN},
    imgproc::{circle, cvt_color, COLOR_BGR2HSV, LINE_8},
    prelude::*,
    videoio::{VideoCapture, VideoCaptureTrait, VideoWriter, VideoWriterTrait, CAP_ANY},
};
use path::Pathfinder;
use remote::CarControl;
use std::thread;

fn main() {
    let test_cap = VideoCapture::from_file("/home/linus/media/track.mp4", 0).unwrap();
    let car = CarControl::new(Car::new(12, 13, 3.6..10.4));
    let clone = car.clone();
    thread::spawn(|| remote::serve(clone));
    Pathfinder::new(car).drive(test_cap);
}

mod motor;
mod path;
mod remote;
#[cfg(test)]
mod tests;

use motor::Car;
use opencv::videoio::{VideoCapture, CAP_ANY};
use path::Pathfinder;
use remote::CarControl;
use std::thread;

fn main() {
    let car = CarControl::new(Car::default());
    let clone = car.clone();
    thread::spawn(|| remote::serve(clone));
    Pathfinder::new(car).drive(VideoCapture::new(0, CAP_ANY).unwrap());
}

fn test() {
    motor::test_speed()
}

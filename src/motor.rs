use rppal::pwm::Pwm;
use std::{ops::Range, time::Duration};
use crate::path::Angle;
use std::thread::sleep;

pub struct Car {
    left: Pwm,
    right: Pwm,
    throttle_range: Range<f32>
}

impl Car {
    pub fn init(&self) {

    }

    pub fn stop(&self) {
        self.left.disable().unwrap();
        self.right.disable().unwrap();
    }

    pub fn start(&self) {
        self.left.set_frequency(50.0, 0.0).unwrap();
        self.right.set_frequency(50.0, 0.0).unwrap();

        self.left.enable().unwrap();
        self.right.enable().unwrap();

        sleep(Duration::from_secs(1));
        self.left.set_frequency(50.0, 1.0).unwrap();
        self.right.set_frequency(50.0, 1.0).unwrap();
        sleep(Duration::from_secs(1));

        self.left.set_frequency(50.0, 0.1).unwrap();
        self.right.set_frequency(50.0, 0.1).unwrap();
    }

    pub fn angle(&self, _angle: Angle) {
        todo!("Implement driving at an angle.")
    }
}

pub fn test() {
    let car = Car {
        left: Pwm::new(rppal::pwm::Channel::Pwm0).unwrap(),
        right: Pwm::new(rppal::pwm::Channel::Pwm1).unwrap(),
        throttle_range: (3.6..10.4)
    };
    car.start();
    sleep(Duration::from_secs(3));
    car.stop();
}

use rppal::pwm::Pwm;
use std::ops::RangeToInclusive;
use crate::path::Angle;

pub struct Car {
    left: Pwm,
    right: Pwm,
    throttle_range: RangeToInclusive<f32>
}

impl Car {
    pub fn stop(&self) {
        self.left.disable().unwrap();
        self.right.disable().unwrap();
    }

    pub fn start(&self) {
        self.left.enable().unwrap();
        self.right.enable().unwrap();
    }

    pub fn angle(&self, angle: Angle) {
        todo!("Implement driving at an angle.")
    }
}

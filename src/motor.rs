use crate::path::Angle;
use rppal::gpio::{Gpio, OutputPin};
use std::thread::sleep;
use std::{ops::Range, time::Duration};

const PWM_0: u8 = 12;
const PWM_1: u8 = 13;
const PWM_FREQ: f64 = 50.0;
const THROTTLE: Range<f32> = 3.6..10.4;

pub type Percent = isize;

pub struct Car {
    left: OutputPin,
    right: OutputPin,
    throttle_range: Range<f32>,
}

impl Car {
    pub fn default() -> Self {
        Self::new(PWM_0, PWM_1, THROTTLE)
    }

    pub fn new(left_pin: u8, right_pin: u8, throttle_range: Range<f32>) -> Self {
        Car {
            left: Gpio::new().unwrap().get(left_pin).unwrap().into_output(),
            right: Gpio::new().unwrap().get(right_pin).unwrap().into_output(),
            throttle_range,
        }
    }

}


pub trait Drivable {
    /// Enables the motors.
    fn enable(&mut self);

    /// Disables the motors.
    fn disable(&mut self);

    /// Stop driving.
    fn stop(&mut self);

    /// Drive forward at given % speed.
    fn forward(&mut self, speed: Percent);

    /// Drive at given angle and give % speed.
    fn angle(&self, _angle: Angle, _speed: Percent);
}

impl Drivable for Car {
    fn enable(&mut self) {}

    fn disable(&mut self) {}

    fn stop(&mut self) {
        self.left.set_pwm_frequency(PWM_FREQ, 0.0).unwrap();
        self.right.set_pwm_frequency(PWM_FREQ, 0.0).unwrap();
    }

    fn forward(&mut self, speed: Percent) {
        let half = (self.throttle_range.end - self.throttle_range.start) / 2.0;
        let neutral = self.throttle_range.start + half;
        let duty_cycle = (neutral + ((half / 100.0) * speed as f32)) / 100.0;
        println!("Duty cycle: {duty_cycle}");
        self.left
            .set_pwm_frequency(PWM_FREQ, duty_cycle as f64)
            .unwrap();
        self.right
            .set_pwm_frequency(PWM_FREQ, duty_cycle as f64)
            .unwrap();
    }

    fn angle(&self, _angle: Angle, _speed: Percent) {
        todo!("Implement driving at an angle.")
    }
}

pub fn test_duty_cycle() {
    let mut car = Car {
        left: Gpio::new().unwrap().get(12).unwrap().into_output(),
        right: Gpio::new().unwrap().get(13).unwrap().into_output(),
        throttle_range: (3.6..10.4),
    };

    car.forward(100);
    sleep(Duration::from_secs(1));
    car.stop();
    sleep(Duration::from_secs(1));

    loop {
        println!("Enter duty cycle (0, 1.0) or 'stop': ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if let Ok(num) = input.trim().parse::<f64>() {
            println!("Setting duty cycle to {num}");
            car.left.set_pwm_frequency(PWM_FREQ, num).unwrap();
            car.right.set_pwm_frequency(PWM_FREQ, num).unwrap();
        } else if "stop" == input.trim() {
            println!("Stopping...");
            break;
        } else {
            println!("Invalid speed: {input}");
        }
    }
}

pub fn test_speed() {
    let mut car = Car {
        left: Gpio::new().unwrap().get(12).unwrap().into_output(),
        right: Gpio::new().unwrap().get(13).unwrap().into_output(),
        throttle_range: (3.6..10.4),
    };

    car.forward(100);
    sleep(Duration::from_secs(1));
    car.stop();
    sleep(Duration::from_secs(1));
    loop {
        println!("Enter speed % or 'stop': ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if let Ok(num) = input.trim().parse::<Percent>() {
            if (num as usize) > 100 {
                println!("Absolute value of {num} > 100");
            } else {
                println!("Setting speed to {num}%");
                car.forward(num);
            }
        } else if "stop" == input.trim() {
            println!("Stopping...");
            break;
        } else {
            println!("Invalid speed: {input}");
        }
    }
}

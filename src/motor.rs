use crate::path::Angle;
use rppal::gpio::{Gpio, OutputPin};
use std::thread::sleep;
use std::{ops::Range, time::Duration};

const PWM_0: u8 = 12;
const PWM_1: u8 = 13;
const PWM_FREQ: f64 = 50.0;
/// Throttle duty cycle range.
const THROTTLE: Range<f64> = 0.04..0.1;

pub type Percent = isize;

pub struct Car {
    left: OutputPin,
    right: OutputPin,
    throttle_range: Range<f64>,
    enabled: bool,
}

impl Car {
    pub fn default() -> Self {
        Self::new(PWM_0, PWM_1, THROTTLE)
    }

    pub fn new(left_pin: u8, right_pin: u8, throttle_range: Range<f64>) -> Self {
        Car {
            left: Gpio::new().unwrap().get(left_pin).unwrap().into_output(),
            right: Gpio::new().unwrap().get(right_pin).unwrap().into_output(),
            throttle_range,
            enabled: false,
        }
    }

    fn duty_cycle_for_speed(&self, speed: &Percent) -> f64 {
        let half = (self.throttle_range.end - self.throttle_range.start) / 2.0;
        let midpoint = self.throttle_range.start + half;
        midpoint + ((half / 100.0) * (*speed) as f64)
    }
}

pub trait Drivable: Send + 'static {
    /// Enables the motors.
    fn enable(&mut self);

    /// Disables the motors.
    /// Motors will not run until enable is called.
    fn disable(&mut self);

    /// Returns true if the motors are enabled.
    fn is_enabled(&self) -> bool;

    /// Drives the left motor with the given duty cycle.
    fn drive_left(&mut self, duty_cycle: f64);

    /// Drives the right motor with the given duty cycle.
    fn drive_right(&mut self, duty_cycle: f64);

    /// Initialises the motors.
    fn init(&mut self);

    /// Stop driving.
    fn stop(&mut self);

    /// Drive forward at given % speed.
    fn forward(&mut self, speed: Percent);

    /// Drive at given angle and give % speed.
    fn angle(&mut self, angle: Angle, speed: Percent);
}

impl Drivable for Car {
    fn enable(&mut self) {
        self.enabled = true;
        self.init();
        println!("Car enabled.");
    }

    fn disable(&mut self) {
        self.stop();
        self.enabled = false;
        println!("Car disabled.");
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn drive_left(&mut self, duty_cycle: f64) {
        if self.enabled {
            self.left.set_pwm_frequency(PWM_FREQ, duty_cycle).unwrap();
        }
    }

    fn drive_right(&mut self, duty_cycle: f64) {
        if self.enabled {
            self.right.set_pwm_frequency(PWM_FREQ, duty_cycle).unwrap();
        }
    }

    fn init(&mut self) {
        self.drive_left(0.01);
        self.drive_right(0.01);
        sleep(Duration::from_secs(2));
    }

    fn stop(&mut self) {
        self.drive_left(0.0);
        self.drive_right(0.0);
    }

    fn forward(&mut self, speed: Percent) {
        let half = (self.throttle_range.end - self.throttle_range.start) / 2.0;
        let midpoint = self.throttle_range.start + half;
        let duty_cycle = midpoint + ((half / 100.0) * speed as f64);
        println!("Duty cycle: {duty_cycle}");
        self.drive_left(duty_cycle);
        self.drive_right(duty_cycle);
    }

    fn angle(&mut self, angle: Angle, speed: Percent) {
        let minor_speed = ((90.0 - angle.abs()) / 90.0) * speed as f64;
        let minor_dc = self.duty_cycle_for_speed(&(minor_speed as isize));
        let major_dc = self.duty_cycle_for_speed(&speed);
        if angle < 0.0 {
            self.drive_left(minor_dc);
            self.drive_right(major_dc);
        } else {
            self.drive_left(major_dc);
            self.drive_right(minor_dc);
        }
    }
}

pub fn test_duty_cycle() {
    let mut car = Car::default();
    car.init();

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
    let mut car = Car::default();
    car.init();

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

pub fn test_angle() {
    let mut car = Car::default();
    car.init();

    loop {
        println!("Enter angle (-90..90) or 'stop': ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if let Ok(num) = input.trim().parse::<Angle>() {
            if (num as usize) > 90 {
                println!("Absolute angle of {num} > 90");
            } else {
                println!("Setting angle to {num}...");
                car.angle(num, 70);
            }
        } else {
            println!("Stopping...");
            break;
        }
    }
}

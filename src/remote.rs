use crate::motor::{Car, Drivable};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{single_middleware, single_pipeline};
use gotham::prelude::*;
use gotham::router::build_router;
use gotham::state::State;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone, StateData)]
pub struct CarControl {
    inner: Arc<Mutex<Car>>,
}

impl CarControl {
    pub fn new(car: Car) -> Self {
        CarControl {
            inner: Arc::new(Mutex::new(car)),
        }
    }

    fn inner(&self) -> MutexGuard<Car> {
        self.inner.lock().unwrap()
    }
}

impl Drivable for CarControl {
    fn enable(&mut self) {
        self.inner().enable();
    }

    fn disable(&mut self) {
        self.inner().disable();
    }

    fn stop(&mut self) {
        self.inner().stop();
    }

    fn forward(&mut self, speed: crate::motor::Percent) {
        self.inner().forward(speed);
    }

    fn angle(&self, _angle: crate::path::Angle, speed: crate::motor::Percent) {
        self.inner().angle(_angle, speed);
    }
}

pub fn enable(mut state: State) -> (State, String) {
    let car = CarControl::borrow_mut_from(&mut state);
    car.enable();

    (state, "Started".to_string())
}

pub fn disable(mut state: State) -> (State, String) {
    let car = CarControl::borrow_mut_from(&mut state);
    car.disable();

    (state, "Stopped".to_string())
}

pub fn serve(car: CarControl) {
    let middleware = StateMiddleware::new(car);

    // create a middleware pipeline from our middleware
    let pipeline = single_middleware(middleware);

    // construct a basic chain from our pipeline
    let (chain, pipelines) = single_pipeline(pipeline);

    // build a router with the chain & pipeline
    let router = build_router(chain, pipelines, |route| {
        route.get("/start").to(enable);
        route.get("/stop").to(disable);
    });

    gotham::start("localhost:80", router).unwrap();
}

use crate::motor::{Car, Drivable};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{single_middleware, single_pipeline};
use gotham::prelude::*;
use gotham::router::build_router;
use gotham::state::State;
use tempfile::NamedTempFile;
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
    println!("Enabling...");
    let car = CarControl::borrow_mut_from(&mut state);
    car.enable();

    (state, "Started".to_string())
}

pub fn disable(mut state: State) -> (State, String) {
    println!("Disabling...");
    let car = CarControl::borrow_mut_from(&mut state);
    car.disable();

    (state, "Stopped".to_string())
}

pub fn serve(car: CarControl) {
    let index_file = NamedTempFile::new().unwrap();
    std::fs::write(&index_file, LANDING_PAGE).unwrap();

    let (chain, pipelines) = single_pipeline(single_middleware(StateMiddleware::new(car)));
    let router = build_router(chain, pipelines, |route| {
        route.get("/").to_file(index_file.path());
        route.post("/start").to(enable);
        route.post("/stop").to(disable);
    });

    gotham::start("0.0.0.0:80", router).unwrap();
}

const LANDING_PAGE: &'static str = "<!DOCTYPE html><html>
<head>
    <h1>Immovable Object Controls</h1>
</head>
<body>
	<div style=\"display:flex\">
        <form action=\"/start\" method=\"post\" target=\"dummy\">
            <button name=\"start\" value=\"start\" style=\"padding:20px;margin:50px\">start</button>
        </form>
        <form action=\"/stop\" method=\"post\" target=\"dummy\">
            <button name=\"stop\" value=\"stop\" style=\"padding:20px;margin:50px\">stop</button>
        </form>
        <iframe name=\"dummy\" id=\"dummy\" hidden/>
	</div>
</body></html>";

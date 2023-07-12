use crate::motor::Drivable;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{single_middleware, single_pipeline};
use gotham::prelude::*;
use gotham::router::build_router;
use gotham::state::State;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(StateData)]
pub struct CarControl<T: Drivable> {
    inner: Arc<Mutex<T>>,
}

impl<T: Drivable> Clone for CarControl<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Drivable> CarControl<T> {
    pub fn new(car: T) -> Self {
        CarControl {
            inner: Arc::new(Mutex::new(car)),
        }
    }

    fn inner(&self) -> MutexGuard<T> {
        self.inner.lock().unwrap()
    }
}

impl<T: Drivable> Drivable for CarControl<T> {
    fn enable(&mut self) {
        self.inner().enable();
    }

    fn disable(&mut self) {
        self.inner().disable();
    }

    fn is_enabled(&self) -> bool {
        self.inner().is_enabled()
    }

    fn drive_left(&mut self, duty_cycle: f64) {
        self.inner().drive_left(duty_cycle);
    }

    fn drive_right(&mut self, duty_cycle: f64) {
        self.inner().drive_right(duty_cycle);
    }

    fn init(&mut self) {
        self.inner().init();
    }

    fn stop(&mut self) {
        self.inner().stop();
    }

    fn forward(&mut self, speed: crate::motor::Percent) {
        self.inner().forward(speed);
    }

    fn angle(&mut self, _angle: crate::path::Angle, speed: crate::motor::Percent) {
        self.inner().angle(_angle, speed);
    }
}

pub fn enable<T: Drivable>(mut state: State) -> (State, String) {
    println!("Enabling...");
    CarControl::<T>::borrow_mut_from(&mut state).enable();
    (state, "Started".to_string())
}

pub fn disable<T: Drivable>(mut state: State) -> (State, String) {
    println!("Disabling...");
    let car = CarControl::<T>::borrow_mut_from(&mut state);
    car.disable();

    (state, "Stopped".to_string())
}

pub fn serve<T: Drivable>(car: CarControl<T>) {
    let landing_page = tempfile::Builder::new().suffix(".html").tempfile().unwrap();
    std::fs::write(&landing_page.path(), LANDING_PAGE_HTML).unwrap();

    let (chain, pipelines) = single_pipeline(single_middleware(StateMiddleware::new(car)));
    let router = build_router(chain, pipelines, |route| {
        route.get("/").to_file(landing_page.path());
        route.post("/start").to(enable::<T>);
        route.post("/stop").to(disable::<T>);
    });

    gotham::start("0.0.0.0:80", router).unwrap();
}

const LANDING_PAGE_HTML: &str = "<!DOCTYPE html><html>
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

use crate::motor::Car;

#[get("/start")]
fn send_start(car: rocket::State<&Car>) {
    car.inner().start();
}

#[get("/stop")]
fn send_stop(car: rocket::State<&Car>) {
    car.inner().stop();
}

#[launch]
fn webp() {
    rocket::build()
        .mount("/", routes![send_start, send_stop])
}

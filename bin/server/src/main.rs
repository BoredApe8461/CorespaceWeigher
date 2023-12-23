/// Web API for interacting with the Consumption Tracker service.
///
/// This API exposes two main endpoints:
/// - `/consumption`: Used to query consumption data associated with a parachain.
/// - `/register`: Used to register a parachain for consumption tracking.
use rocket_cors::CorsOptions;
use routes::register::register_para;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CorsOptions::default().to_cors().unwrap())
        .mount("/", routes![register_para])
}

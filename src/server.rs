use csv::ReaderBuilder;
use std::fs::File;
use rocket_cors::CorsOptions;

mod parachains;

mod shared;
use shared::*;

mod types;
use types::*;

#[macro_use]
extern crate rocket;
use rocket::http::Status;

#[get("/consumption/<relay>/<para_id>")]
fn consumption(relay: &str, para_id: ParaId) -> Result<String, Status> {
    let para = parachains::parachain(relay.into(), para_id).ok_or(Status::NotFound)?;

    let file = File::open(file_path(para)).map_err(|_| Status::NotFound)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    let weight_consumptions: Vec<WeightConsumption> = rdr
        .deserialize::<WeightConsumption>()
        .filter_map(|result| result.ok())
        .collect();

    serde_json::to_string(&weight_consumptions).map_err(|_| Status::InternalServerError)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CorsOptions::default().to_cors().unwrap())
        .mount("/", routes![consumption])
}

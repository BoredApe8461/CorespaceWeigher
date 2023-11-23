use csv::ReaderBuilder;
use std::fs::File;

mod parachains;

mod shared;
use shared::*;

mod types;
use types::*;

#[macro_use]
extern crate rocket;

#[get("/consumption/<relay>/<para_id>")]
fn consumption(relay: &str, para_id: ParaId) -> String {
    if let Some(para) = parachains::parachain(relay.into(), para_id) {
        // TODO: don't unwrap
        let file = File::open(file_path(para)).unwrap();
        let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

        let records: Vec<String> = rdr
            .deserialize::<WeightConsumption>()
            .map(|result| serde_json::to_string(&result.unwrap()).unwrap())
            .collect();

        format!("{:?}", records)
    } else {
        format!("Para not found")
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![consumption])
}

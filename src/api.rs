/// Web API for interacting with the Consumption Tracker service.
///
/// This API exposes two main endpoints:
/// - `/consumption`: Used to query consumption data associated with a parachain.
/// - `/register`: Used to register a parachain for consumption tracking.
use csv::ReaderBuilder;
use rocket::{http::Status, response::Responder, serde::json::Json, Request, Response};
use rocket_cors::CorsOptions;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
};

mod shared;
use shared::*;

mod types;
use types::*;

#[macro_use]
extern crate rocket;

#[derive(Debug)]
enum Error {
    /// Cannot register an already registered parachain.
    AlreadyRegistered,
    /// Tried to get the consumption of a parachain that is not registered.
    NotRegistered,
    /// Indicates that the consumption data for the parachain was not found.
    ///
    /// This should be only encountered if the consumption file has not been generated yet,
    /// which is only possible if the parachain has been registered within the last few seconds.
    ConsumptionDataNotFound,
    /// The stored data is invalid. This should never really happen.
    InvalidData,
    /// Failed to find the parachains data. This isn't a user error, but a bug in the code itself.
    ParasDataNotFound,
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let body = format!("Error: {:?}", self);
        Response::build()
            .status(Status::InternalServerError)
            .sized_body(body.len(), std::io::Cursor::new(body))
            .ok()
    }
}

/// Query the consumption data of a parachain.
///
/// This will return an error in case there is no data associated with the specific parachain.
#[get("/consumption/<relay>/<para_id>?<page>&<page_size>")]
fn consumption(
    relay: &str,
    para_id: ParaId,
    page: Option<u32>,
    page_size: Option<u32>,
) -> Result<String, Error> {
    let para = parachain(relay.into(), para_id).ok_or(Error::NotRegistered)?;

    let file = File::open(file_path(para)).map_err(|_| Error::ConsumptionDataNotFound)?;
    let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

    let (page, page_size) = (page.unwrap_or_default(), page_size.unwrap_or(u32::MAX));


    let weight_consumptions: Vec<WeightConsumption> = rdr
        .deserialize::<WeightConsumption>()
        .filter_map(|result| result.ok())
        .skip((page.saturating_add(page_size)) as usize)
        .take(page_size as usize)
        .collect();

    serde_json::to_string(&weight_consumptions).map_err(|_| Error::InvalidData)
}

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<para>")]
fn register_para(para: Json<Parachain>) -> Result<String, Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(PARACHAINS)
        .map_err(|_| Error::ParasDataNotFound)?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|_| Error::InvalidData)?;

    let mut paras: Vec<Parachain> =
        serde_json::from_str(&content).map_err(|_| Error::InvalidData)?;

    if parachain(para.relay_chain.clone(), para.para_id.clone()).is_some() {
        return Err(Error::AlreadyRegistered);
    }

    paras.push(para.into_inner());
    let json_data = serde_json::to_string_pretty(&paras).expect("Failed to serialize");

    file.set_len(0).expect("Failed to truncate file");
    file.seek(std::io::SeekFrom::Start(0))
        .expect("Failed to seek to the beginning");

    file.write_all(json_data.as_bytes()).unwrap();

    Ok(format!(""))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(CorsOptions::default().to_cors().unwrap())
        .mount("/", routes![consumption, register_para])
}

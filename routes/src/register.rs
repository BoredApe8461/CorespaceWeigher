use crate::*;
use rocket::{post, serde::json::Json};
use shared::{parachain, PARACHAINS};
use std::{
    fs::OpenOptions,
    io::{Read, Seek, Write},
};
use types::Parachain;

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<para>")]
pub fn register_para(para: Json<Parachain>) -> Result<String, Error> {
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

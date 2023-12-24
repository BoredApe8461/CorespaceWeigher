// This file is part of RegionX.
//
// RegionX is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// RegionX is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with RegionX.  If not, see <https://www.gnu.org/licenses/>.

use crate::*;
use rocket::{post, serde::json::Json};
use shared::{parachain, PARACHAINS};
use std::{
	fs::{File, OpenOptions},
	io::{Read, Seek, Write},
};
use types::Parachain;

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<para>")]
pub fn register_para(para: Json<Parachain>) -> Result<(), Error> {
	let mut file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(PARACHAINS)
		.map_err(|_| Error::ParasDataNotFound)?;

	let mut content = String::new();
	file.read_to_string(&mut content).map_err(|_| Error::InvalidData)?;

	let mut paras: Vec<Parachain> =
		serde_json::from_str(&content).map_err(|_| Error::InvalidData)?;

	if parachain(para.relay_chain.clone(), para.para_id).is_some() {
		return Err(Error::AlreadyRegistered);
	}

	paras.push(para.into_inner());

	if let Err(err) = update_paras_file(&mut file, paras) {
		log::error!(
			target: LOG_TARGET,
			"Failed to register para: {:?}",
			err
		);
	}

	Ok(())
}

fn update_paras_file(file: &mut File, paras: Vec<Parachain>) -> Result<(), String> {
	let json_data = serde_json::to_string_pretty(&paras).map_err(|_| "Failed to serialize")?;

	file.set_len(0).map_err(|_| "Failed to truncate file")?;
	file.seek(std::io::SeekFrom::Start(0))
		.map_err(|_| "Failed to seek to the beginning")?;

	file.write_all(json_data.as_bytes()).map_err(|_| "Failed to write into file")?;

	Ok(())
}

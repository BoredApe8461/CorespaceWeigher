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

use crate::config::config;
use std::{
	fs::{File, OpenOptions},
	io::{Read, Seek, Write},
};
use types::{ParaId, Parachain, RelayChain};

pub fn registered_paras() -> Vec<Parachain> {
	let mut registry = get_registry();
	let mut content = String::new();

	// If this fails it simply means that the registered parachains is still empty.
	let _ = registry.read_to_string(&mut content);
	let paras: Vec<Parachain> = serde_json::from_str(&content).expect("Failed to serialize");

	paras
}

pub fn registered_para(relay_chain: RelayChain, para_id: ParaId) -> Option<Parachain> {
	registered_paras()
		.iter()
		.find(|para| para.relay_chain == relay_chain && para.para_id == para_id)
		.cloned()
}

pub fn update_registry(paras: Vec<Parachain>) -> Result<(), String> {
	let mut registry = get_registry();
	let json_data = serde_json::to_string_pretty(&paras).map_err(|_| "Failed to serialize")?;

	registry.set_len(0).map_err(|_| "Failed to truncate file")?;
	registry
		.seek(std::io::SeekFrom::Start(0))
		.map_err(|_| "Failed to seek to the beginning")?;

	registry
		.write_all(json_data.as_bytes())
		.map_err(|_| "Failed to write into file")?;

	Ok(())
}

fn get_registry() -> File {
	match OpenOptions::new().read(true).write(true).create(true).open(config().registry) {
		Ok(file) => file,
		Err(_) => init_registry(),
	}
}

pub fn init_registry() -> File {
	let mut registry =
		File::create(config().registry).expect("Failed to create registered para file");
	// An empty vector
	registry.write_all(b"[]").expect("Failed to write into registered para file");

	registry
}

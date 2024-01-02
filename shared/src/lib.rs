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

use csv::WriterBuilder;
use std::{
	fs::{File, OpenOptions},
	io::{Read, Seek, Write},
};
use types::{ParaId, Parachain, RelayChain, WeightConsumption};

const LOG_TARGET: &str = "shared";

pub const CONFIG_FILE: &str = "config.toml";

#[derive(serde::Deserialize)]
struct Config {
	output_directory: String,
	registry: String,
}

pub fn registered_parachains() -> Vec<Parachain> {
	let mut file = File::open(registry_file_path()).expect("Couldn't find the registry file");

	let mut content = String::new();
	if file.read_to_string(&mut content).is_ok() {
		let paras: Vec<Parachain> = serde_json::from_str(&content).unwrap_or_default();
		paras
	} else {
		Default::default()
	}
}

pub fn parachain(relay_chain: RelayChain, para_id: ParaId) -> Option<Parachain> {
	registered_parachains()
		.iter()
		.find(|para| para.relay_chain == relay_chain && para.para_id == para_id)
		.cloned()
}

pub fn registry_file_path() -> String {
	config().registry
}

pub fn output_file_path(para: Parachain) -> String {
	format!("{}/{}-{}.csv", config().output_directory, para.relay_chain, para.para_id)
}

pub fn round_to(number: f32, decimals: i32) -> f32 {
	let factor = 10f32.powi(decimals);
	(number * factor).round() / factor
}

pub fn write_consumption(
	para: Parachain,
	consumption: WeightConsumption,
) -> Result<(), std::io::Error> {
	log::info!(
		target: LOG_TARGET,
		"Writing weight consumption for Para {}-{} for block: #{}",
		para.relay_chain, para.para_id, consumption.block_number
	);

	let output_file_path = output_file_path(para);
	let file = OpenOptions::new().create(true).append(true).open(output_file_path)?;

	let mut wtr = WriterBuilder::new().from_writer(file);

	// The data is stored in the sequence described at the beginning of the file.
	wtr.write_record(&[
		// Block number:
		consumption.block_number.to_string(),
		// Timestamp:
		consumption.timestamp.to_string(),
		// Reftime consumption:
		consumption.ref_time.normal.to_string(),
		consumption.ref_time.operational.to_string(),
		consumption.ref_time.mandatory.to_string(),
		// Proof size:
		consumption.proof_size.normal.to_string(),
		consumption.proof_size.operational.to_string(),
		consumption.proof_size.mandatory.to_string(),
	])?;

	wtr.flush()
}

pub fn registered_paras() -> Vec<Parachain> {
	let mut registry = get_registry();
	let mut content = String::new();

	// If this fails it simply means that the registered parachains is still empty.
	let _ = registry.read_to_string(&mut content);
	let paras: Vec<Parachain> = serde_json::from_str(&content).expect("Failed to serialize");

	paras
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

// There isn't a good reason to use this other than for testing.
#[cfg(feature = "test-utils")]
pub fn reset_mock_environment() {
	let config = config();

	// Reset the registered paras file:
	let _registry = init_registry();

	// Remove the output files:
	let _ = std::fs::create_dir(config.output_directory.clone());

	for entry in
		std::fs::read_dir(config.output_directory).expect("Failed to read output directory")
	{
		let entry = entry.expect("Failed to ready entry");
		let path = entry.path();
		if path.is_file() {
			std::fs::remove_file(path).expect("Failed to remove consumption data")
		}
	}
}

fn config() -> Config {
	let config_str = std::fs::read_to_string("config.toml").expect("Failed to read config file");
	toml::from_str(&config_str).expect("Failed to parse config file")
}

fn get_registry() -> File {
	match OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(registry_file_path())
	{
		Ok(file) => file,
		Err(_) => init_registry(),
	}
}

fn init_registry() -> File {
	let mut registry =
		File::create(registry_file_path()).expect("Failed to create registered para file");
	// An empty vector
	registry.write_all(b"[]").expect("Failed to write into registered para file");

	registry
}

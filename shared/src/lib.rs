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
use std::fs::OpenOptions;
use types::{Parachain, WeightConsumption};

pub mod registry;

const LOG_TARGET: &str = "shared";

pub const CONFIG_FILE: &str = "config.toml";

#[derive(serde::Deserialize)]
struct Config {
	output_directory: String,
	registry: String,
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

// There isn't a good reason to use this other than for testing.
#[cfg(feature = "test-utils")]
pub fn reset_mock_environment() {
	let config = config();

	// Reset the registered paras file:
	let _registry = registry::init_registry();

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

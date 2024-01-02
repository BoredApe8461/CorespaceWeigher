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

use crate::{config::config, LOG_TARGET};
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::{File, OpenOptions};
use types::{Parachain, WeightConsumption};

pub fn get_consumption(para: Parachain) -> Result<Vec<WeightConsumption>, &'static str> {
	let file = File::open(output_file_path(para)).map_err(|_| "Consumption data not found")?;
	let mut rdr = ReaderBuilder::new().has_headers(false).from_reader(file);

	let consumption: Vec<WeightConsumption> = rdr
		.deserialize::<WeightConsumption>()
		.filter_map(|result| result.ok())
		.collect();

	Ok(consumption)
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

fn output_file_path(para: Parachain) -> String {
	format!("{}/{}-{}.csv", config().output_directory, para.relay_chain, para.para_id)
}

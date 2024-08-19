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

use crate::{config::output_directory, LOG_TARGET};
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::{File, OpenOptions};
use types::{Parachain, WeightConsumption};

pub fn get_consumption(
	para: Parachain,
	rpc_index: Option<usize>,
) -> Result<Vec<WeightConsumption>, &'static str> {
	let file =
		File::open(output_file_path(para, rpc_index)).map_err(|_| "Consumption data not found")?;
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
	rpc_index: Option<usize>,
) -> Result<(), std::io::Error> {
	log::info!(
		target: LOG_TARGET,
		"{}-{} - Writing weight consumption for block: #{}",
		para.relay_chain, para.para_id, consumption.block_number
	);

	let output_file_path = output_file_path(para, rpc_index);
	let file = OpenOptions::new().create(true).append(true).open(output_file_path)?;

	let mut wtr = WriterBuilder::new().from_writer(file);

	// The data is stored in the sequence described at the beginning of the file.
	wtr.write_record(&consumption.to_csv())?;

	wtr.flush()
}

pub fn write_batch_consumption(
	para: Parachain,
	consumption: Vec<WeightConsumption>,
) -> Result<(), std::io::Error> {
	log::info!(
		target: LOG_TARGET,
		"{}-{} - Writing batch weight consumption.",
		para.relay_chain, para.para_id
	);

	let output_file_path = output_file_path(para, None);
	let file = OpenOptions::new().create(true).append(true).open(output_file_path)?;

	let mut wtr = WriterBuilder::new().from_writer(file);

	// TODO: add a to_csv function
	consumption.iter().try_for_each(|entry| {
		// The data is stored in the sequence described at the beginning of the file.
		wtr.write_record(&entry.to_csv())
	})?;

	wtr.flush()
}

pub fn delete_consumption(para: Parachain, rpc_index: usize) {
	log::info!(
		target: LOG_TARGET,
		"{}-{} - Deleting weight consumption.",
		para.relay_chain, para.para_id
	);

	let output_file_path = output_file_path(para, Some(rpc_index));
	match std::fs::remove_file(output_file_path.clone()) {
		Ok(_) => {
			log::info!(
				target: LOG_TARGET,
				"{} Deleted successfully",
				output_file_path
			);
		},
		Err(e) => {
			log::error!(
				target: LOG_TARGET,
				"{} Failed to delete: {:?}",
				output_file_path, e
			);
		},
	}
}

fn output_file_path(para: Parachain, rpc_index: Option<usize>) -> String {
	format!("{}/{}-{}.csv", output_directory(rpc_index), para.relay_chain, para.para_id)
}

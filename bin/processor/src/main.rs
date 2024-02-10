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

use shared::{
	config::config,
	consumption::{delete_consumption, get_consumption, write_batch_consumption},
	registry::registered_paras,
};
use std::collections::BTreeMap;
use types::WeightConsumption;

const LOG_TARGET: &str = "processor";

fn main() {
	env_logger::init();

	let outputs = config().outputs;
	let paras = registered_paras();

	paras.iter().for_each(|para| {
		let mut processed = BTreeMap::new();

		log::info!(
			target: LOG_TARGET,
			"{}-{} - Processing consumption.",
			para.relay_chain,
			para.para_id,
		);

		(0..outputs).for_each(|output_index| {
			let consumption = if let Ok(data) = get_consumption(para.clone(), Some(output_index)) {
				data
			} else {
				log::error!(
					target: LOG_TARGET,
					"{}-{} - Failed to get consumption.",
					para.relay_chain,
					para.para_id,
				);
				vec![]
			};

			consumption.into_iter().for_each(|data| {
				processed.entry(data.block_number).or_insert(data);
			});
		});

		let processed: Vec<WeightConsumption> = processed.values().cloned().collect();

		log::info!(
			target: LOG_TARGET,
			"{}-{} - Writing processed consumption. Total blocks tracked: {}",
			para.relay_chain,
			para.para_id,
			processed.len()
		);

		if let Err(e) = write_batch_consumption(para.clone(), processed) {
			log::error!(
				target: LOG_TARGET,
				"{}-{} - Failed to write batch consumption: {:?}",
				para.relay_chain,
				para.para_id,
				e,
			);

			return;
		}

		(0..outputs).for_each(|output_index| delete_consumption(para.clone(), output_index));
	});
}

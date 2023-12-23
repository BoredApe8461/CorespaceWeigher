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

use crate::Error;
use csv::ReaderBuilder;
use rocket::get;
use shared::{file_path, parachain};
use std::fs::File;
use types::{ParaId, WeightConsumption};

/// Query the consumption data of a parachain.
///
/// This will return an error in case there is no data associated with the specific parachain.
#[get("/consumption/<relay>/<para_id>?<page>&<page_size>")]
pub fn consumption(
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

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
use rocket::get;
use shared::{consumption::get_consumption, registry::registered_para};
use types::{ParaId, Timestamp, WeightConsumption};

/// Query the consumption data of a parachain.
///
/// This will return an error in case there is no data associated with the specific parachain.
#[get("/consumption/<relay>/<para_id>?<start>&<end>&<page>&<page_size>")]
pub fn consumption(
	relay: &str,
	para_id: ParaId,
	start: Option<Timestamp>,
	end: Option<Timestamp>,
	page: Option<u32>,
	page_size: Option<u32>,
) -> Result<String, Error> {
	let para = registered_para(relay.into(), para_id).ok_or(Error::NotRegistered)?;

	let (page, page_size) = (page.unwrap_or_default(), page_size.unwrap_or(u32::MAX));
	let (start, end) = (start.unwrap_or_default(), end.unwrap_or(Timestamp::MAX));

	let weight_consumptions: Vec<WeightConsumption> = get_consumption(para)
		.map_err(|_| Error::ConsumptionDataNotFound)?
		.into_iter()
		.filter(|consumption| consumption.timestamp >= start && consumption.timestamp <= end)
		.skip(page.saturating_mul(page_size) as usize)
		.take(page_size as usize)
		.collect();

	serde_json::to_string(&weight_consumptions).map_err(|_| Error::InvalidData)
}

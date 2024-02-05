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
use chrono::NaiveDateTime;
use rocket::{
	form,
	form::{FromFormField, ValueField},
	get,
};
use shared::{consumption::get_consumption, registry::registered_para};
use std::collections::HashMap;
use types::{DispatchClassConsumption, ParaId, Timestamp, WeightConsumption};

#[derive(Clone, Debug, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub enum Grouping {
	BlockNumber,
	Day,
	Month,
	Year,
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for Grouping {
	fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
		match field.value {
			"day" => Ok(Grouping::Day),
			"month" => Ok(Grouping::Month),
			"year" => Ok(Grouping::Year),
			_ => Err(form::Error::validation("invalid Grouping").into()),
		}
	}
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AggregatedData {
	/// The aggregated ref_time consumption over all the dispatch classes.
	pub ref_time: DispatchClassConsumption,
	/// The aggregated proof size over all dispatch classes.
	pub proof_size: DispatchClassConsumption,
	pub count: usize,
}

/// Query the consumption data of a parachain.
///
/// This will return an error in case there is no data associated with the specific parachain.
#[get("/consumption/<relay>/<para_id>?<start>&<end>&<page>&<page_size>&<grouping>")]
pub fn consumption(
	relay: &str,
	para_id: ParaId,
	start: Option<Timestamp>,
	end: Option<Timestamp>,
	page: Option<u32>,
	page_size: Option<u32>,
	grouping: Option<Grouping>,
) -> Result<String, Error> {
	let para = registered_para(relay.into(), para_id).ok_or(Error::NotRegistered)?;

	let (page, page_size) = (page.unwrap_or_default(), page_size.unwrap_or(u32::MAX));
	let (start, end) = (start.unwrap_or_default(), end.unwrap_or(Timestamp::MAX));

	// By default query the consumption that was collected from rpc index 0.
	let weight_consumptions: Vec<WeightConsumption> = get_consumption(para, 0)
		.map_err(|_| Error::ConsumptionDataNotFound)?
		.into_iter()
		.filter(|consumption| consumption.timestamp >= start && consumption.timestamp <= end)
		.skip(page.saturating_mul(page_size) as usize)
		.take(page_size as usize)
		.collect();

	let grouping = grouping.unwrap_or(Grouping::BlockNumber);

	let grouped: HashMap<String, AggregatedData> = group_consumption(weight_consumptions, grouping);

	serde_json::to_string(&grouped).map_err(|_| Error::InvalidData)
}

pub fn group_consumption(
	weight_consumptions: Vec<WeightConsumption>,
	grouping: Grouping,
) -> HashMap<String, AggregatedData> {
	weight_consumptions.iter().fold(HashMap::new(), |mut acc, datum| {
		let key = get_aggregation_key(datum.clone(), grouping);
		let entry = acc.entry(key).or_insert_with(Default::default);

		entry.ref_time.normal += datum.ref_time.normal;
		entry.ref_time.operational += datum.ref_time.operational;
		entry.ref_time.mandatory += datum.ref_time.mandatory;

		entry.proof_size.normal += datum.proof_size.normal;
		entry.proof_size.operational += datum.proof_size.operational;
		entry.proof_size.mandatory += datum.proof_size.mandatory;

		entry.count += 1;

		acc
	})
}

fn get_aggregation_key(datum: WeightConsumption, grouping: Grouping) -> String {
	let datetime =
		NaiveDateTime::from_timestamp_opt((datum.timestamp / 1000) as i64, 0).unwrap_or_default();

	match grouping {
		Grouping::BlockNumber => datum.block_number.to_string(),
		Grouping::Day => datetime.format("%Y-%m-%d").to_string(),
		Grouping::Month => datetime.format("%Y-%m").to_string(),
		Grouping::Year => datetime.format("%Y").to_string(),
	}
}

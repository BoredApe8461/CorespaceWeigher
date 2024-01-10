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
use polkadot_core_primitives::BlockNumber;
use rocket::{post, serde::json::Json};
use shared::{
	config::config,
	current_timestamp,
	payment::validate_registration_payment,
	registry::{registered_para, registered_paras, update_registry},
};
use types::{ParaId, RelayChain};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ExtendSubscriptionData {
	/// The parachain which is getting its subscription extended.
	pub para: (RelayChain, ParaId),
	/// The block in which the payment occurred for the specific parachain.
	pub payment_block_number: BlockNumber,
}

/// Extend the subscription of a parachain for resource utilization tracking.
#[post("/extend_subscription", data = "<data>")]
pub async fn extend_subscription(data: Json<ExtendSubscriptionData>) -> Result<(), Error> {
	let (relay_chain, para_id) = data.para.clone();

	log::info!(
		target: LOG_TARGET,
		"Attempting to extend subscription for para: {}:{}",
		relay_chain, para_id
	);

	let mut para = registered_para(relay_chain.clone(), para_id).ok_or(Error::NotRegistered)?;

	if let Some(payment_info) = config().payment_info {
		validate_registration_payment(para.clone(), payment_info, data.payment_block_number)
			.await
			.map_err(Error::PaymentValidationError)?;
	}

	let mut paras = registered_paras();

	if let Some(para) = paras.iter_mut().find(|p| **p == para) {
		para.last_payment_timestamp = current_timestamp();
	} else {
		return Err(Error::NotRegistered);
	}

	para.last_payment_timestamp = current_timestamp();

	if let Err(err) = update_registry(paras) {
		log::error!(
			target: LOG_TARGET,
			"Failed to extend subscription for para: {:?}",
			err
		);
	}

	Ok(())
}

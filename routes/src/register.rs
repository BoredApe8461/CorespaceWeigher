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
	chaindata,
	config::config,
	current_timestamp,
	payment::validate_registration_payment,
	registry::{registered_para, registered_paras, update_registry},
};
use types::{ParaId, RelayChain};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationData {
	/// The parachain getting registered.
	pub para: (RelayChain, ParaId),
	/// The block in which the payment occurred for the specific parachain.
	///
	/// In free mode (where payment is not required), this is ignored and can be `None`.
	/// Otherwise, it should contain a valid block number.
	pub payment_block_number: Option<BlockNumber>,
}

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<registration_data>")]
pub async fn register_para(registration_data: Json<RegistrationData>) -> Result<(), Error> {
	let (relay_chain, para_id) = registration_data.para.clone();

	log::info!(
		target: LOG_TARGET,
		"{}-{} - Attempting to register para",
		relay_chain, para_id
	);

	let mut paras = registered_paras();

	if registered_para(relay_chain.clone(), para_id).is_some() {
		return Err(Error::AlreadyRegistered);
	}

	let mut para = chaindata::get_para(relay_chain, para_id).map_err(Error::ChainDataError)?;

	let subscription_duration = if let Some(payment_info) = config().payment_info {
		let payment_block_number =
			registration_data.payment_block_number.ok_or(Error::PaymentRequired)?;

		validate_registration_payment(para.clone(), payment_info.clone(), payment_block_number)
			.await
			.map_err(Error::PaymentValidationError)?;

		payment_info.subscription_duration
	} else {
		Default::default()
	};

	para.expiry_timestamp = current_timestamp() + subscription_duration;

	paras.push(para.clone());

	if let Err(err) = update_registry(paras) {
		log::error!(
			target: LOG_TARGET,
			"{}-{} - Failed to register para: {:?}",
			para.relay_chain,
			para.para_id,
			err
		);
	} else {
		#[cfg(not(debug_assertions))]
		shared::init_tracker();
	}

	Ok(())
}

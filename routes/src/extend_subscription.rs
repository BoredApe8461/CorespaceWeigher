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

use crate::{
	register::polkadot::runtime_types::{
		frame_system::pallet::Call as SystemCall, pallet_balances::pallet::Call as BalancesCall,
		pallet_utility::pallet::Call as UtilityCall,
	},
	*,
};
use parity_scale_codec::Encode;
use polkadot_core_primitives::BlockNumber;
use rocket::{post, serde::json::Json};
use shared::{
	config::{config, PaymentInfo},
	current_timestamp,
	registry::{registered_para, registered_paras, update_registry},
};
use subxt::{
	backend::rpc::{rpc_params, RpcClient},
	blocks::Block,
	utils::H256,
	OnlineClient, PolkadotConfig,
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

	let mut paras = registered_paras();

	if registered_para(relay_chain.clone(), para_id).is_some() {
		return Err(Error::NotRegistered);
	}

	Ok(())
}

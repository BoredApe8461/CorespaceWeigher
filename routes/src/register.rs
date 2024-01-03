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
use polkadot_core_primitives::Block;
use rocket::{post, serde::json::Json};
use shared::{
	config::config,
	registry::{registered_para, registered_paras, update_registry},
};
use sp_runtime::generic::SignedBlock;
use subxt::{
	backend::rpc::{rpc_params, RpcClient},
	utils::H256,
};
use types::{BlockNumber, Parachain};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(crate = "rocket::serde")]
pub struct PaymentInfo {
	/// The block number in which the payment occurred.
	block_number: BlockNumber,
	/// The extrinsic that pays for the subscription.
	extrinsic_hash: H256,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationData {
	/// The parachain getting registered.
	para: Parachain,
	/// Optional payment-related information.
	///
	/// In free mode (where payment is not required), this is ignored and can be `None`.
	/// Otherwise, it should contain valid `PaymentInfo` details.
	payment_info: Option<PaymentInfo>,
}

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<registration_data>")]
pub async fn register_para(registration_data: Json<RegistrationData>) -> Result<(), Error> {
	let para = registration_data.para.clone();

	let mut paras = registered_paras();

	if registered_para(para.relay_chain.clone(), para.para_id).is_some() {
		return Err(Error::AlreadyRegistered);
	}

	// If not free mode check if the specified extrinsic contains the correct remark and a transfer.
	if !config().free_mode {
		let info = registration_data.payment_info.clone().ok_or(Error::PaymentRequired)?;

		if let Some(payment_rpc_url) = config().payment_rpc_url {
			check_registration_payment(info, payment_rpc_url).await?;
		} else {
			log::error!(
				target: LOG_TARGET,
				"Free mode is disabled, but the payment_rpc_url is not set in the config",
			);
		}
	}

	paras.push(para);

	if let Err(err) = update_registry(paras) {
		log::error!(
			target: LOG_TARGET,
			"Failed to register para: {:?}",
			err
		);
	}

	Ok(())
}

/*
curl -X POST http://127.0.0.1:8000/register_para -H "Content-Type: application/json" -d '{
	"para": {
		"name": "Acala",
		"rpc_url": "wss://acala-rpc.dwellir.com",
		"para_id": 2005,
		"relay_chain": "Polkadot"
	},
	"payment_info": {
		"block_number": 18881079,
		"extrinsic_hash": "0x31f1a85007834dd8cc138be74da4f626db752b139043d252c047d401e5f63207"
	}
}'
*/

async fn check_registration_payment(
	payment_info: PaymentInfo,
	payment_rpc_url: String,
) -> Result<(), Error> {
	if let Ok(rpc_client) = RpcClient::from_url(&payment_rpc_url).await {
		let params = rpc_params![Some(payment_info.block_number)];
		let block_hash: H256 = rpc_client.request("chain_getBlockHash", params).await.unwrap();

		let params = rpc_params![block_hash];
		let maybe_rpc_response: Option<serde_json::Value> =
			rpc_client.request("chain_getBlock", params).await.unwrap();
		let rpc_response = maybe_rpc_response.unwrap();

		let opaque_block: SignedBlock<Block> = serde_json::from_value(rpc_response).unwrap();

		println!("{:?}", opaque_block);
	}

	Ok(())
}

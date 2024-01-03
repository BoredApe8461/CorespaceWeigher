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
use polkadot_core_primitives::{Block, BlockNumber};
use rocket::{post, serde::json::Json};
use shared::{
	config::{config, PaymentInfo},
	registry::{registered_para, registered_paras, update_registry},
};
use sp_runtime::generic::SignedBlock;
use subxt::{
	backend::rpc::{rpc_params, RpcClient},
	tx::TxPayload,
	utils::H256,
	OnlineClient, PolkadotConfig,
};
use types::Parachain;

type AccountId = H256;

#[subxt::subxt(runtime_metadata_path = "../artifacts/metadata.scale")]
mod polkadot {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(crate = "rocket::serde")]
pub struct Receipt {
	/// The block number in which the payment occurred.
	block_number: BlockNumber,
	/// The account that pays for the subscription.
	payer: AccountId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(crate = "rocket::serde")]
pub struct RegistrationData {
	/// The parachain getting registered.
	para: Parachain,
	/// Optional payment-related information.
	///
	/// In free mode (where payment is not required), this is ignored and can be `None`.
	/// Otherwise, it should contain valid `Receipt` details.
	receipt: Option<Receipt>,
}

/// Register a parachain for resource utilization tracking.
#[post("/register_para", data = "<registration_data>")]
pub async fn register_para(registration_data: Json<RegistrationData>) -> Result<(), Error> {
	let para = registration_data.para.clone();

	let mut paras = registered_paras();

	if registered_para(para.relay_chain.clone(), para.para_id).is_some() {
		return Err(Error::AlreadyRegistered);
	}

	if let Some(payment_info) = config().payment_info {
		let receipt = registration_data.receipt.clone().ok_or(Error::PaymentRequired)?;

		validate_registration_payment(payment_info, receipt).await?;
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
	"receipt": {
		"block_number": 18881079,
		"payer": "126X27SbhrV19mBFawys3ovkyBS87SGfYwtwa8J2FjHrtbmA"
	}
}'
*/

async fn validate_registration_payment(
	payment_info: PaymentInfo,
	receipt: Receipt,
) -> Result<(), Error> {
	if let Ok(rpc_client) = RpcClient::from_url(&payment_info.rpc_url).await {
		let params = rpc_params![Some(receipt.block_number)];
		let block_hash: H256 = rpc_client.request("chain_getBlockHash", params).await.unwrap();

		let params = rpc_params![block_hash];
		let maybe_rpc_response: Option<serde_json::Value> =
			rpc_client.request("chain_getBlock", params).await.unwrap();
		let rpc_response = maybe_rpc_response.unwrap();

		let opaque_block: SignedBlock<Block> = serde_json::from_value(rpc_response).unwrap();
		let opaque_extrinsics = opaque_block.block.extrinsics;

		let payment = payment_extrinsic(payment_info, receipt).await?;

		Ok(())
	} else {
		Err(Error::PaymentValidationFailed)
	}
}

async fn payment_extrinsic(payment_info: PaymentInfo, receipt: Receipt) -> Result<Vec<u8>, Error> {
	if let Ok(online_client) = OnlineClient::<PolkadotConfig>::from_url(payment_info.rpc_url).await
	{
		let transfer = polkadot::tx()
			.balances()
			.transfer_keep_alive(payment_info.receiver.into(), payment_info.cost);

		let remark = polkadot::tx().system().remark(vec![]);

		let transfer_encoded = transfer.encode_call_data(&online_client.metadata()).unwrap();
		let remark_encoded = remark.encode_call_data(&online_client.metadata()).unwrap();

		//let batch = polkadot::tx().utility().batch_all(vec![transfer_encoded, remark_encoded]);

		Ok(transfer_encoded)
	} else {
		Err(Error::PaymentValidationFailed)
	}
}

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
	registry::{registered_para, registered_paras, update_registry},
};
use subxt::{
	backend::rpc::{rpc_params, RpcClient},
	utils::{AccountId32, H256},
	OnlineClient, PolkadotConfig,
};
use types::Parachain;

#[subxt::subxt(runtime_metadata_path = "../artifacts/metadata.scale")]
mod polkadot {}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Receipt {
	/// The block number in which the payment occurred.
	block_number: BlockNumber,
	/// The account that pays for the subscription.
	payer: AccountId32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

	log::info!(
		target: LOG_TARGET,
		"Attempting to register para: {}:{}",
		para.relay_chain, para.para_id
	);

	let mut paras = registered_paras();

	if registered_para(para.relay_chain.clone(), para.para_id).is_some() {
		return Err(Error::AlreadyRegistered);
	}

	if let Some(payment_info) = config().payment_info {
		let receipt = registration_data.receipt.clone().ok_or(Error::PaymentRequired)?;

		validate_registration_payment(para.clone(), payment_info, receipt).await?;
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
	para: Parachain,
	payment_info: PaymentInfo,
	receipt: Receipt,
) -> Result<(), Error> {
	let rpc_client = RpcClient::from_url(&payment_info.rpc_url.clone())
		.await
		.map_err(|_| Error::PaymentValidationFailed)?;

	let params = rpc_params![Some(receipt.block_number)];
	// TODO: ensure that the specified block is finalized.
	let block_hash: H256 = rpc_client.request("chain_getBlockHash", params).await.unwrap();

	let api = OnlineClient::<PolkadotConfig>::from_url(payment_info.rpc_url.clone())
		.await
		.map_err(|_| Error::PaymentValidationFailed)?;

	let block = api.blocks().at(block_hash).await.map_err(|_| Error::PaymentValidationFailed)?;
	let payment = opaque_payment_extrinsic(para, payment_info).await?;

	let extrinsics = block.extrinsics().await.unwrap();
	let extrinsics: Vec<Vec<u8>> = extrinsics
		.iter()
		.filter_map(|ext| {
			ext.as_ref().ok().and_then(|e| e.as_root_extrinsic::<polkadot::Call>().ok())
		})
		.map(|ext| ext.encode())
		.collect();

	if extrinsics.contains(&payment.encode()) {
		Ok(())
	} else {
		Err(Error::PaymentNotFound)
	}
}

async fn opaque_payment_extrinsic(
	para: Parachain,
	payment_info: PaymentInfo,
) -> Result<polkadot::Call, Error> {
	if let Ok(cost) = payment_info.cost.parse::<u128>() {
		let transfer_call = polkadot::Call::Balances(BalancesCall::transfer_keep_alive {
			dest: payment_info.receiver.into(),
			value: cost,
		});

		let remark = format!("{}:{}", para.relay_chain, para.para_id).as_bytes().to_vec();
		let remark_call = polkadot::Call::System(SystemCall::remark { remark });

		let batch_call = polkadot::Call::Utility(UtilityCall::batch_all {
			calls: vec![transfer_call, remark_call],
		});

		Ok(batch_call)
	} else {
		log::error!(
			target: LOG_TARGET,
			"Failed to parse cost",
		);
		Err(Error::PaymentValidationFailed)
	}
}

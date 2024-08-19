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

//! # Consumption Tracker
//!
//! This is the main source file for the Consumption Tracker binary.
//!
//! ## Overview
//!
//! The program is designed to fetch weight utilization data from a predefined set
//! of parachains. The obtained weight information is then stored in the `out`
//! directory as multiple CSV files.
//!
//! ## Output Structure
//!
//! Each parachain has its own dedicated output file, and these files are updated
//! every time a new block is finalized and the weight consumption data is
//! successfully queried.
//!
//! ## Data structure
//!
//! The data stored is the 2D weight consumption per each dispatch class.
//! The data is stored in the CSV file within the following sequence:
//!
//! | block_number | timestamp             | normal_dispatch_ref_time  | operational_dispatch_ref_time | mandatory_dispatch_ref_time | normal_proof_size | operational_proof_size  | mandatory_proof_size  |
//! |--------------|-----------------------|---------------------------|-------------------------------|-----------------------------|-------------------|-------------------------|-----------------------|
//! | ...          | ...                   | ...                       | ...                           | ...                         | ...               | ...                     | ...                   |
//!
//! The percentages themselves are stored by representing them as decimal numbers;
//! for example, 50.5% is stored as 0.505 with a precision of three decimals.

const LOG_TARGET: &str = "tracker";

use clap::Parser;
use shared::{consumption::write_consumption, registry::registered_paras, round_to};
use subxt::{blocks::Block, utils::H256, OnlineClient, PolkadotConfig};
use types::{Parachain, Timestamp, WeightConsumption};

mod cli;

#[subxt::subxt(runtime_metadata_path = "../../artifacts/metadata.scale")]
mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let args = cli::Args::parse();

	// Asynchronously subscribes to follow the latest finalized block of each parachain
	// and continuously fetches the weight consumption.
	let tasks: Vec<_> = registered_paras()
		.into_iter()
		.map(|para| {
			tokio::spawn(async move { track_weight_consumption(para, args.rpc_index).await })
		})
		.collect();

	for task in tasks {
		task.await.expect("Failed to track consumption");
	}

	Ok(())
}

async fn track_weight_consumption(para: Parachain, rpc_index: usize) {
	let Some(rpc) = para.rpcs.get(rpc_index) else {
		log::error!(
			target: LOG_TARGET,
			"{}-{} - doesn't have an rpc with index: {}",
			para.relay_chain, para.para_id, rpc_index,
		);
		return;
	};

	log::info!("{}-{} - Starting to track consumption.", para.relay_chain, para.para_id);
	let result = OnlineClient::<PolkadotConfig>::from_url(rpc).await;

	if let Ok(api) = result {
		if let Err(err) = track_blocks(api, para.clone(), rpc_index).await {
			log::error!(
				target: LOG_TARGET,
				"{}-{} - Failed to track new block: {:?}",
				para.relay_chain,
				para.para_id,
				err
			);
		}
	} else {
		log::error!(
			target: LOG_TARGET,
			"{}-{} - Failed to create online client: {:?}",
			para.relay_chain,
			para.para_id,
			result
		);
	}
}

async fn track_blocks(
	api: OnlineClient<PolkadotConfig>,
	para: Parachain,
	rpc_index: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	log::info!(
		target: LOG_TARGET,
		"{}-{} - Subsciribing to finalized blocks",
		para.relay_chain,
		para.para_id
	);

	let mut blocks_sub = api
		.blocks()
		.subscribe_finalized()
		.await
		.map_err(|_| "Failed to subscribe to finalized blocks")?;

	// Wait for new finalized blocks, then fetch and output the weight consumption accordingly.
	while let Some(Ok(block)) = blocks_sub.next().await {
		note_new_block(api.clone(), para.clone(), rpc_index, block).await?;
	}

	Ok(())
}

async fn note_new_block(
	api: OnlineClient<PolkadotConfig>,
	para: Parachain,
	rpc_index: usize,
	block: Block<PolkadotConfig, OnlineClient<PolkadotConfig>>,
) -> Result<(), Box<dyn std::error::Error>> {
	let block_number = block.header().number;

	let timestamp = timestamp_at(api.clone(), block.hash()).await?;
	let consumption = weight_consumption(api, block_number, timestamp).await?;

	write_consumption(para, consumption, Some(rpc_index))?;

	Ok(())
}

async fn weight_consumption(
	api: OnlineClient<PolkadotConfig>,
	block_number: u32,
	timestamp: Timestamp,
) -> Result<WeightConsumption, Box<dyn std::error::Error>> {
	let weight_query = polkadot::storage().system().block_weight();
	let weight_consumed = api
		.storage()
		.at_latest()
		.await?
		.fetch(&weight_query)
		.await?
		.ok_or("Failed to query consumption")?;

	let weight_limit_query = polkadot::constants().system().block_weights();
	let weight_limit = api.constants().at(&weight_limit_query)?;

	let proof_limit = weight_limit.max_block.proof_size;
	// NOTE: This will be the same for all parachains within the same network until elastic scaling
	// is enabled.
	let ref_time_limit = weight_limit.max_block.ref_time;

	let normal_ref_time = weight_consumed.normal.ref_time;
	let operational_ref_time = weight_consumed.operational.ref_time;
	let mandatory_ref_time = weight_consumed.mandatory.ref_time;

	let normal_proof_size = weight_consumed.normal.proof_size;
	let operational_proof_size = weight_consumed.operational.proof_size;
	let mandatory_proof_size = weight_consumed.mandatory.proof_size;

	let consumption = WeightConsumption {
		block_number,
		timestamp,
		ref_time: (
			round_to(normal_ref_time as f32 / ref_time_limit as f32, 3),
			round_to(operational_ref_time as f32 / ref_time_limit as f32, 3),
			round_to(mandatory_ref_time as f32 / ref_time_limit as f32, 3),
		)
			.into(),
		proof_size: (
			round_to(normal_proof_size as f32 / proof_limit as f32, 3),
			round_to(operational_proof_size as f32 / proof_limit as f32, 3),
			round_to(mandatory_proof_size as f32 / proof_limit as f32, 3),
		)
			.into(),
	};

	Ok(consumption)
}

async fn timestamp_at(
	api: OnlineClient<PolkadotConfig>,
	block_hash: H256,
) -> Result<Timestamp, Box<dyn std::error::Error>> {
	let timestamp_query = polkadot::storage().timestamp().now();

	let timestamp = api
		.storage()
		.at(block_hash)
		.fetch(&timestamp_query)
		.await?
		.ok_or("Failed to query consumption")?;

	Ok(timestamp)
}

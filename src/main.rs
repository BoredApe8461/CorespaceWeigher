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
//! | block_number | normal_dispatch_ref_time | operational_dispatch_ref_time | mandatory_dispatch_ref_time | normal_proof_size | operational_proof_size | mandatory_proof_size |
//! |--------------|---------------------------|-------------------------------|-----------------------------|-------------------|-------------------------|-----------------------|
//! | ...          | ...                       | ...                           | ...                         | ...               | ...                     | ...                   |
//!
//! The percentages themselves are stored by representing them as decimal numbers;
//! for example, 50.5% is stored as 0.505 with a precision of three decimals.

use csv::WriterBuilder;
use std::fs::OpenOptions;
use subxt::{OnlineClient, PolkadotConfig};

mod shared;
use shared::*;

pub mod types;
use types::*;

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Asynchronously subscribes to follow the latest finalized block of each parachain
    // and continuously fetches the weight consumption.
    let tasks: Vec<_> = parachains()
        .into_iter()
        .map(|para| tokio::spawn(async move { track_weight_consumption(para).await }))
        .collect();

    for task in tasks {
        task.await.expect("Failed to track consumption");
    }

    Ok(())
}

async fn track_weight_consumption(para: Parachain) {
    if let Ok(api) = OnlineClient::<PolkadotConfig>::from_url(&para.rpc_url).await {
        let mut blocks_sub = api
            .blocks()
            .subscribe_finalized()
            .await
            .expect("Failed to subscribe to finalized blocks");

        // Wait for new finalized blocks, then fetch and output the weight consumption accordingly.
        while let Some(Ok(block)) = blocks_sub.next().await {
            let block_number = block.header().number;
            if let Ok(consumption) = weight_consumption(api.clone(), block_number).await {
                let _ = write_consumption(para.clone(), consumption);
            }
        }
    }
}

fn write_consumption(
    para: Parachain,
    consumption: WeightConsumption,
) -> Result<(), std::io::Error> {
    let file_path = file_path(para);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_path)?;

    let mut wtr = WriterBuilder::new().from_writer(file);

    // The data is stored in the sequence described at the beginning of the file.
    wtr.write_record(&[
        // Block number:
        consumption.block_number.to_string(),
        // Reftime consumption:
        consumption.ref_time.normal.to_string(),
        consumption.ref_time.operational.to_string(),
        consumption.ref_time.mandatory.to_string(),
        // Proof size:
        consumption.proof_size.normal.to_string(),
        consumption.proof_size.operational.to_string(),
        consumption.proof_size.mandatory.to_string(),
    ])?;

    wtr.flush()
}

async fn weight_consumption(
    api: OnlineClient<PolkadotConfig>,
    block_number: u32,
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

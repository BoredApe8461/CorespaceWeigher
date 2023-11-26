use csv::WriterBuilder;
use std::fs::OpenOptions;
use subxt::{OnlineClient, PolkadotConfig};

mod shared;
use shared::*;

pub mod types;
use types::*;

mod parachains;
use parachains::*;

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    wtr.write_record(&[
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

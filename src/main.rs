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
        consumption.normal.to_string(),
        consumption.operational.to_string(),
        consumption.mandatory.to_string(),
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

    // NOTE: This will be the same for all parachains within the same network until elastic scaling
    // is enabled.
    let weight_limit = weight_limit.max_block.ref_time;

    let normal_consumed = weight_consumed.normal.ref_time;
    let operational_consumed = weight_consumed.operational.ref_time;
    let mandatory_consumed = weight_consumed.mandatory.ref_time;

    let consumption = WeightConsumption {
        block_number,
        normal: round_to_2_decimals(normal_consumed as f32 / weight_limit as f32),
        operational: round_to_2_decimals(operational_consumed as f32 / weight_limit as f32),
        mandatory: round_to_2_decimals(mandatory_consumed as f32 / weight_limit as f32),
    };

    Ok(consumption)
}

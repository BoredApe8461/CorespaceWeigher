use subxt::{OnlineClient, PolkadotConfig};
use csv::WriterBuilder;
use std::fs::OpenOptions;

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
        task.await.unwrap();
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
            if let Ok(consumption) = weight_consumption(api.clone()).await {
                // println!("{} - {}: \n{}", block.header().number, para.name, consumption);
                write_consumption(para.clone(), block.header().number, consumption);
            }
        }
    }
}

fn write_consumption(para: Parachain, block_number: u32, consumption: WeightConsumption) {
    let file_path = format!("out/{:?}-{}.csv", para.relay_chain, para.para_id);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(file_path)
        .expect("Failed to open CSV file");
    
    let mut wtr = WriterBuilder::new().from_writer(file);

    wtr.write_record(&[
        block_number.to_string(),
        consumption.normal.to_string(),
        consumption.operational.to_string(),
        consumption.mandatory.to_string(),
    ])
    .unwrap(); // TODO: don't unwrap

    wtr.flush().unwrap();
}

async fn weight_consumption(
    api: OnlineClient<PolkadotConfig>,
) -> Result<WeightConsumption, Box<dyn std::error::Error>> {
    let weight_query = polkadot::storage().system().block_weight();
    let weight_consumed = api
        .storage()
        .at_latest()
        .await?
        .fetch(&weight_query)
        .await?
        .ok_or("Failed")?;

    let weight_limit_query = polkadot::constants().system().block_weights();
    let weight_limit = api.constants().at(&weight_limit_query)?;

    // NOTE: This will be the same for all parachains within the same network until elastic scaling
    // is enabled.
    let weight_limit = weight_limit.max_block.ref_time;

    let normal_consumed = weight_consumed.normal.ref_time;
    let operational_consumed = weight_consumed.operational.ref_time;
    let mandatory_consumed = weight_consumed.mandatory.ref_time;

    let consumption = WeightConsumption {
        normal: normal_consumed as f32 / weight_limit as f32,
        operational: operational_consumed as f32 / weight_limit as f32,
        mandatory: mandatory_consumed as f32 / weight_limit as f32,
    };

    Ok(consumption)
}

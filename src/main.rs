use subxt::{OnlineClient, PolkadotConfig};

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
        .map(|para| tokio::spawn(async move { log_weight_consumption(para).await }))
        .collect();

    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

async fn log_weight_consumption(para: Parachain) {
    if let Ok(consumption) = weight_consumption(&para.rpc_url).await {
        println!("{}: \n{}", para.name, consumption);
    }
}

async fn weight_consumption(
    rpc_url: &str,
) -> Result<WeightConsumption, Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await?;

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

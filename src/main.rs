use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
pub mod polkadot {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url("wss://moonbeam.public.blastapi.io").await?;

    let weight_query = polkadot::storage().system().block_weight();
    let weight_consumed = api
        .storage()
        .at_latest()
        .await?
        .fetch(&weight_query)
        .await?
        .ok_or("Failed")?;

    let weight_limit_query =  polkadot::constants().system().block_weights();
    let weight_limit = api.constants().at(&weight_limit_query)?;

    let weight_limit = weight_limit.max_block.ref_time;

    let normal_consumed = weight_consumed.normal.ref_time;
    let operational_consumed = weight_consumed.operational.ref_time;
    let mandatory_consumed = weight_consumed.mandatory.ref_time;

    println!("Normal consumed: {}", normal_consumed);
    println!("Operational consumed: {}", operational_consumed);
    println!("Mandatory consumed: {}", mandatory_consumed);

    println!("Weight limit: {}", weight_limit);

    println!("Normal consumption: {}", normal_consumed as f32 / weight_limit as f32);
    println!("Operational consumption: {}", operational_consumed as f32 / weight_limit as f32);
    println!("Mandatory consumption: {}", mandatory_consumed as f32 / weight_limit as f32);

    Ok(())
}

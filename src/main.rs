use subxt::{OnlineClient, Config, SubstrateConfig, PolkadotConfig};
use parity_scale_codec::Decode;
use subxt_metadata::Metadata;

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
pub mod polkadot {}

pub static METADATA: &[u8] = include_bytes!("../artifacts/metadata.scale");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url("wss://rpc.polkadot.io").await?;

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

    let normal_limit = weight_limit.per_class.normal.max_total.unwrap().ref_time;
    let operational_limit = weight_limit.per_class.operational.max_total.unwrap().ref_time;
    //let mandatory_limit = weight_limit.per_class.mandatory.max_total.unwrap().ref_time;

    let normal_consumed = weight_consumed.normal.ref_time;
    let operational_consumed = weight_consumed.operational.ref_time;

    println!("Normal consumed: {}", normal_consumed);
    println!("Operational consumed: {}", operational_consumed);

    println!("Normal limit: {}", normal_limit);
    println!("Operational limit: {}", operational_limit);
    //println!("{:?}", mandatory_limit);

    println!("Normal consumption: {}", normal_consumed as f32 / normal_limit as f32);
    println!("Operational consumption: {}", operational_consumed / operational_limit);

    Ok(())
}

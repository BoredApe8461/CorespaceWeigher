use subxt::{OnlineClient, PolkadotConfig};

#[subxt::subxt(runtime_metadata_path = "artifacts/metadata.scale")]
pub mod polkadot {}

struct Parachain {
    name: String,
    rpc_url: String,
    para_id: u32,
}

#[derive(Debug)]
struct WeightConsumption {
    /// The percentage of the weight used by user submitted extrinsics compared to the
    /// maximum potential.
    normal: f32,
    /// The percentage of the weight used by user operational dispatches compared to the
    /// maximum potential.
    operational: f32,
    /// The percentage of the weight used by the mandatory tasks of a parachain compared 
    /// to the maximum potential.
    mandatory: f32,
}

impl std::fmt::Display for WeightConsumption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n\tNormal consumption: {}", self.normal)?;
        write!(f, "\n\tOperational consumption: {}", self.operational)?;
        write!(f, "\n\tMandatory consumption: {}", self.mandatory)?;
        Ok(())
    }
}

fn parachains() -> Vec<Parachain> {    
    vec![
    Parachain { name: "Moonbeam".to_string(), rpc_url: "wss://moonbeam.public.blastapi.io".to_string(), para_id: 2004 },
    Parachain { name: "Acala".to_string(), rpc_url: "wss://acala-polkadot.api.onfinality.io/public-ws".to_string(), para_id: 2000 },
    Parachain { name: "Astar".to_string(), rpc_url: "wss://astar.api.onfinality.io/public-ws".to_string(), para_id: 2006 },
    ]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    for para in parachains() {
        let consumption = weight_consumption(para.rpc_url.as_str()).await?;

        print!("{}: \n{}", para.name, consumption);
    }

    Ok(())
}

async fn weight_consumption(rpc_url: &str) -> Result<WeightConsumption, Box<dyn std::error::Error>> {
    let api = OnlineClient::<PolkadotConfig>::from_url(rpc_url).await?;

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

    let consumption = WeightConsumption {
        normal: normal_consumed as f32 / weight_limit as f32,
        operational: operational_consumed as f32 / weight_limit as f32,
        mandatory: mandatory_consumed as f32 / weight_limit as f32, 
    };

    Ok(consumption)
}

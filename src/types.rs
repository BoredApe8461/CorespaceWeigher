use serde::{Deserialize, Serialize};
use std::fmt;

pub type ParaId = u32;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RelayChain {
    Polkadot,
    Kusama,
}

impl fmt::Display for RelayChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelayChain::Polkadot => write!(f, "Polkadot"),
            RelayChain::Kusama => write!(f, "Kusama"),
        }
    }
}

impl From<&str> for RelayChain {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "polkadot" => RelayChain::Polkadot,
            "kusama" => RelayChain::Kusama,
            _ => panic!("Invalid relay chain: {}", s),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Parachain {
    /// Name of the parachain.
    pub name: String,
    /// The rpc url endpoint from where we can query the weight consumption.
    //
    // TODO: instead of having only one rpc url specified there should be a fallback.
    pub rpc_url: String,
    /// The `ParaId` of the parachain.
    pub para_id: ParaId,
    /// The relay chain that the parachain is using for block validation.
    pub relay_chain: RelayChain,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeightConsumption {
    /// The block number for which the weight consumption is related to.
    pub block_number: u32,
    /// The percentage of the weight used by user submitted extrinsics compared to the
    /// maximum potential.
    pub normal: f32,
    /// The percentage of the weight used by user operational dispatches compared to the
    /// maximum potential.
    pub operational: f32,
    /// The percentage of the weight used by the mandatory tasks of a parachain compared
    /// to the maximum potential.
    pub mandatory: f32,
}

impl fmt::Display for WeightConsumption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n\tNormal consumption: {}", self.normal)?;
        write!(f, "\n\tOperational consumption: {}", self.operational)?;
        write!(f, "\n\tMandatory consumption: {}", self.mandatory)?;
        Ok(())
    }
}

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

use serde::{Deserialize, Serialize};
use std::fmt;

pub type ParaId = u32;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
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
	/// The ref_time consumption over all the dispatch classes.
	pub ref_time: DispatchClassConsumption,
	/// The proof size over all dispatch classes.
	pub proof_size: DispatchClassConsumption,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DispatchClassConsumption {
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

/// A shorthand for converting a tuple of `f32`s into `DispatchClassConsumption`.
///
/// The order in which the values need to be provided is: `normal`, `operational`, `mandatory`.
impl From<(f32, f32, f32)> for DispatchClassConsumption {
	fn from(value: (f32, f32, f32)) -> Self {
		DispatchClassConsumption { normal: value.0, operational: value.1, mandatory: value.2 }
	}
}

impl fmt::Display for WeightConsumption {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "\n\tNormal ref_time consumption: {}", self.ref_time.normal)?;
		write!(f, "\n\tOperational ref_time consumption: {}", self.ref_time.operational)?;
		write!(f, "\n\tMandatory ref_time consumption: {}", self.ref_time.mandatory)?;

		write!(f, "\n\tNormal proof size: {}", self.proof_size.normal)?;
		write!(f, "\n\tOperational proof size: {}", self.proof_size.operational)?;
		write!(f, "\n\tMandatory proof size: {}", self.proof_size.mandatory)?;
		Ok(())
	}
}

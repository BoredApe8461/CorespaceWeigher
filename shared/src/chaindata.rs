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
use crate::config::config;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read};
use types::{ParaId, Parachain, RelayChain};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Relay {
	id: RelayChain,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Rpc {
	url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
struct ChainData {
	pub name: String,
	pub para_id: ParaId,
	pub relay: Relay,
	pub rpcs: Vec<Rpc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ChainDataError {
	ParaNotFound,
}

impl From<String> for ChainDataError {
	fn from(v: String) -> Self {
		match v.as_str() {
			"ParaNotFound" => Self::ParaNotFound,
			_ => panic!("UnknownError"),
		}
	}
}

/// Get the rpcs of a parachain.
pub fn get_para(relay: RelayChain, para_id: ParaId) -> Result<Parachain, ChainDataError> {
	let mut file = File::open(config().chaindata).expect("ChainData not found");
	let mut content = String::new();

	file.read_to_string(&mut content).expect("Failed to load chaindata");
	let chaindata: Vec<ChainData> = serde_json::from_str(&content).expect("Failed to serialize");

	let index = chaindata
		.iter()
		.position(|para| para.para_id == para_id && para.relay == Relay { id: relay.clone() })
		.ok_or(ChainDataError::ParaNotFound)?;

	let para_chaindata = chaindata.get(index).expect("We just found the index; qed");

	let rpcs: Vec<String> = para_chaindata.rpcs.clone().into_iter().map(|rpc| rpc.url).collect();

	let para = Parachain {
		relay_chain: relay,
		para_id,
		name: para_chaindata.name.clone(),
		rpcs,
		expiry_timestamp: Default::default(),
	};

	Ok(para)
}

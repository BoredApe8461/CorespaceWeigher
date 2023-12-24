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

use std::{fs::File, io::Read};
use types::{ParaId, Parachain, RelayChain};

pub const CONFIG_FILE: &str = "config.toml";

#[derive(serde::Deserialize)]
struct Config {
	output_directory: String,
	parachains_file: String,
}

pub fn parachains() -> Vec<Parachain> {
	let mut file = File::open(parachains_file_path()).expect("Hardcoded path is known good; qed");

	let mut content = String::new();
	if file.read_to_string(&mut content).is_ok() {
		let paras: Vec<Parachain> = serde_json::from_str(&content).unwrap_or_default();
		paras
	} else {
		Default::default()
	}
}

#[allow(dead_code)]
pub fn parachain(relay_chain: RelayChain, para_id: ParaId) -> Option<Parachain> {
	parachains()
		.iter()
		.find(|para| para.relay_chain == relay_chain && para.para_id == para_id)
		.cloned()
}

pub fn parachains_file_path() -> String {
	let config_str = std::fs::read_to_string("config.toml").expect("Failed to read config file");

	let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

	config.parachains_file
}

pub fn output_file_path(para: Parachain) -> String {
	let config_str = std::fs::read_to_string("config.toml").expect("Failed to read config file");

	let config: Config = toml::from_str(&config_str).expect("Failed to parse config file");

	format!("{}/{}-{}.csv", config.output_directory, para.relay_chain, para.para_id)
}

#[allow(dead_code)]
pub fn round_to(number: f32, decimals: i32) -> f32 {
	let factor = 10f32.powi(decimals);
	(number * factor).round() / factor
}

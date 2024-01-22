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

use subxt::utils::AccountId32;
use types::Timestamp;

const CONFIG_FILE: &str = "config.toml";

#[derive(serde::Deserialize)]
pub struct PaymentInfo {
	/// The rpc url from the chain where the payment is required to occur.
	pub rpc_url: String,
	/// The account that the payment should be sent to.
	pub receiver: AccountId32,
	/// The cost of the payment.
	//
	// Defined as a `String` since the `toml` crate has issues parsing `u128`.
	pub cost: String,
	/// This defines the duration that a single subscription payment will cover.
	pub subscription_duration: Timestamp,
}

#[derive(serde::Deserialize)]
pub struct Config {
	pub output_directory: String,
	pub registry: String,
	pub chaindata: String,
	pub payment_info: Option<PaymentInfo>,
}

pub fn config() -> Config {
	let config_str = std::fs::read_to_string(CONFIG_FILE).expect("Failed to read config file");
	toml::from_str(&config_str).expect("Failed to parse config file")
}

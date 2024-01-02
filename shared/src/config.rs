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

const CONFIG_FILE: &str = "config.toml";

#[derive(serde::Deserialize)]
pub struct Config {
	pub output_directory: String,
	pub registry: String,
}

pub fn config() -> Config {
	let config_str = std::fs::read_to_string(CONFIG_FILE).expect("Failed to read config file");
	toml::from_str(&config_str).expect("Failed to parse config file")
}

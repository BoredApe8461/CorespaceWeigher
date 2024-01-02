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

pub mod config;
pub mod consumption;
pub mod registry;

use crate::config::config;

const LOG_TARGET: &str = "shared";

pub fn round_to(number: f32, decimals: i32) -> f32 {
	let factor = 10f32.powi(decimals);
	(number * factor).round() / factor
}

// There isn't a good reason to use this other than for testing.
#[cfg(feature = "test-utils")]
pub fn reset_mock_environment() {
	let config = config();

	// Reset the registered paras file:
	let _registry = registry::init_registry();

	// Remove the output files:
	let _ = std::fs::create_dir(config.output_directory.clone());

	for entry in
		std::fs::read_dir(config.output_directory).expect("Failed to read output directory")
	{
		let entry = entry.expect("Failed to ready entry");
		let path = entry.path();
		if path.is_file() {
			std::fs::remove_file(path).expect("Failed to remove consumption data")
		}
	}
}

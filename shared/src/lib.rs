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

use std::{
	process::Command,
	time::{SystemTime, UNIX_EPOCH},
};
use types::Timestamp;

pub mod chaindata;
pub mod config;
pub mod consumption;
pub mod payment;
pub mod registry;

#[cfg(feature = "test-utils")]
use crate::config::output_directory;

const LOG_TARGET: &str = "shared";

/// Rounds a number to a fixed number of decimals.
pub fn round_to(number: f32, decimals: i32) -> f32 {
	let factor = 10f32.powi(decimals);
	(number * factor).round() / factor
}

/// Returns the current time since UNIX EPOCH.
pub fn current_timestamp() -> Timestamp {
	// It is fine to use `unwrap_or_default` since the current time will never be before the UNIX
	// EPOCH.
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}


pub fn init_tracker() {
	let output = Command::new("./scripts/init.sh").output().expect("Failed to execute command");

	if output.status.success() {
		log::info!("Successfully reinitalized tracker");
	} else {
		let stderr = String::from_utf8_lossy(&output.stderr);
		log::info!("Failed to reinitialize tracker: {:?}", stderr);
	}
}

// There isn't a good reason to use this other than for testing.
#[cfg(feature = "test-utils")]
pub fn reset_mock_environment() {
	// Reset the registered paras file:
	let _registry = registry::init_registry();

	let output_path = output_directory(None);
	// Remove the output files:
	let _ = std::fs::create_dir(output_path.clone());

	for entry in std::fs::read_dir(output_path).expect("Failed to read output directory") {
		let entry = entry.expect("Failed to ready entry");
		let path = entry.path();
		if path.is_file() {
			std::fs::remove_file(path).expect("Failed to remove consumption data")
		}
	}
}

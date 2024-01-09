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

#[cfg(test)]
use maplit::hashmap;
use scopeguard::guard;
use shared::{consumption::write_consumption, registry::update_registry, reset_mock_environment, current_timestamp};
use std::collections::HashMap;
use types::{ParaId, Parachain, RelayChain, RelayChain::*, WeightConsumption};

#[derive(Default)]
pub struct MockEnvironment {
	pub weight_consumptions: HashMap<Parachain, Vec<WeightConsumption>>,
}

impl MockEnvironment {
	pub fn new() -> Self {
		// Start with an empty environment.
		reset_mock_environment();

		// Initialize some mock data:
		let mock = MockEnvironment { weight_consumptions: mock_consumption() };

		for (para, weight_consumptions) in &mock.weight_consumptions {
			weight_consumptions.iter().for_each(|consumption| {
				write_consumption(para.clone(), consumption.clone())
					.expect("Failed to write conusumption data");
			});
		}

		let _ = update_registry(mock.weight_consumptions.keys().cloned().collect());

		mock
	}

	pub fn execute_with<R>(&self, execute: impl FnOnce() -> R) -> R {
		let _guard = guard((), |_| {
			// Reset the environment once we are complete with the test.
			reset_mock_environment();
		});

		execute()
	}
}

pub fn mock_consumption() -> HashMap<Parachain, Vec<WeightConsumption>> {
	hashmap! {
		mock_para(Polkadot, 2000) => vec![
			WeightConsumption {
				block_number: 1,
				timestamp: 0,
				ref_time: (0.5, 0.3, 0.2).into(),
				proof_size: (0.5, 0.3, 0.2).into(),
			},
			WeightConsumption {
				block_number: 2,
				timestamp: 6,
				ref_time: (0.1, 0.4, 0.2).into(),
				proof_size: (0.2, 0.3, 0.3).into(),
			},
			WeightConsumption {
				block_number: 3,
				timestamp: 12,
				ref_time: (0.0, 0.2, 0.4).into(),
				proof_size: (0.1, 0.0, 0.3).into(),
			},
			WeightConsumption {
				block_number: 4,
				timestamp: 18,
				ref_time: (0.1, 0.0, 0.4).into(),
				proof_size: (0.2, 0.1, 0.3).into(),
			},
		],
		mock_para(Polkadot, 2005) => vec![
			WeightConsumption {
				block_number: 1,
				timestamp: 0,
				ref_time: (0.8, 0.0, 0.1).into(),
				proof_size: (0.6, 0.2, 0.1).into(),
			},
		],
	}
}

pub fn mock_para(relay: RelayChain, para_id: ParaId) -> Parachain {
	Parachain {
		name: format!("{}-{}", relay, para_id),
		rpc_url: format!("wss://{}-{}.com", relay, para_id),
		para_id,
		relay_chain: relay,
		last_payment_timestamp: 0
	}
}

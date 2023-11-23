/// File containing the list of all parachains that we query the weight consumption for.
use crate::types::{Parachain, RelayChain::*};

pub fn parachains() -> Vec<Parachain> {
    vec![
        Parachain {
            name: "Acala".to_string(),
            rpc_url: "wss://acala-polkadot.api.onfinality.io/public-ws".to_string(),
            para_id: 2000,
            relay_chain: Polkadot,
        },
        Parachain {
            name: "Moonbeam".to_string(),
            rpc_url: "wss://moonbeam.public.blastapi.io".to_string(),
            para_id: 2004,
            relay_chain: Polkadot,
        },
        Parachain {
            name: "Astar".to_string(),
            rpc_url: "wss://astar.api.onfinality.io/public-ws".to_string(),
            para_id: 2006,
            relay_chain: Polkadot,
        },
    ]
}

/// File containing the list of all parachains that we query the weight consumption for.
use crate::types::{ParaId, Parachain, RelayChain};

pub fn parachains() -> Vec<Parachain> {
    vec![
        Parachain {
            name: "AssetHub".to_string(),
            rpc_url: "wss://statemint.api.onfinality.io/public-ws".to_string(),
            para_id: 1000,
            relay_chain: RelayChain::Polkadot,
        },
        Parachain {
            name: "Acala".to_string(),
            rpc_url: "wss://acala-polkadot.api.onfinality.io/public-ws".to_string(),
            para_id: 2000,
            relay_chain: RelayChain::Polkadot,
        },
        Parachain {
            name: "Moonbeam".to_string(),
            rpc_url: "wss://moonbeam.public.blastapi.io".to_string(),
            para_id: 2004,
            relay_chain: RelayChain::Polkadot,
        },
        Parachain {
            name: "Astar".to_string(),
            rpc_url: "wss://astar.api.onfinality.io/public-ws".to_string(),
            para_id: 2006,
            relay_chain: RelayChain::Polkadot,
        },
        Parachain {
            name: "Phala".to_string(),
            rpc_url: "wss://phala.api.onfinality.io/public-ws".to_string(),
            para_id: 2035,
            relay_chain: RelayChain::Polkadot,
        },
    ]
}

pub fn parachain(relay_chain: RelayChain, para_id: ParaId) -> Option<Parachain> {
    parachains()
        .iter()
        .find(|para| para.relay_chain == relay_chain && para.para_id == para_id)
        .cloned()
}

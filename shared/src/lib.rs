use std::{fs::File, io::Read};
use types::{ParaId, Parachain, RelayChain};

pub const PARACHAINS: &str = "parachains.json";

pub fn parachains() -> Vec<Parachain> {
    let mut file = File::open(PARACHAINS).expect("Hardcoded path is known good; qed");

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

pub fn file_path(para: Parachain) -> String {
    format!("out/{}-{}.csv", para.relay_chain, para.para_id)
}

#[allow(dead_code)]
pub fn round_to(number: f32, decimals: i32) -> f32 {
    let factor = 10f32.powi(decimals);
    (number * factor).round() / factor
}

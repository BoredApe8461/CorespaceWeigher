use crate::types::*;

pub fn file_path(para: Parachain) -> String {
    format!("out/{}-{}.csv", para.relay_chain, para.para_id)
}

pub fn round_to_2_decimals(number: f32) -> f32 {
    let factor = 100.0;
    (number * factor).round() / factor
}

use crate::types::*;

pub fn file_path(para: Parachain) -> String {
    format!("out/{}-{}.csv", para.relay_chain, para.para_id)
}

pub fn round_to(number: f32) -> f32 {
    let factor = 10f32.powf(number);
    (number * factor).round() / factor
}

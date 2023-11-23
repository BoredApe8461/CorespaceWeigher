use crate::types::*;

pub fn file_path(para: Parachain) -> String {
    format!("out/{}-{}.csv", para.relay_chain, para.para_id)
}

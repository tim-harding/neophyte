use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct ModeChange {
    pub mode: String,
    pub mode_idx: u64,
}

impl ModeChange {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            mode: parse_string(iter.next()?)?,
            mode_idx: parse_u64(iter.next()?)?,
        })
    }
}

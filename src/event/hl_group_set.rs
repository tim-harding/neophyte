use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct HlGroupSet {
    pub name: String,
    pub hl_id: u64,
}

impl HlGroupSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            name: parse_string(iter.next()?)?,
            hl_id: parse_u64(iter.next()?)?,
        })
    }
}

use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Clone, Copy, Debug)]
pub struct GridCursorGoto {
    pub grid: u64,
    pub row: u64,
    pub column: u64,
}

impl GridCursorGoto {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            grid: parse_u64(iter.next()?)?,
            row: parse_u64(iter.next()?)?,
            column: parse_u64(iter.next()?)?,
        })
    }
}

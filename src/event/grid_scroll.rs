use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct GridScroll {
    pub grid: u64,
    pub top: u64,
    pub bot: u64,
    pub left: u64,
    pub right: u64,
    pub rows: u64,
    pub cols: u64,
}

impl TryFrom<Value> for GridScroll {
    type Error = GridScrollParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter();
            Some(Self {
                grid: parse_u64(iter.next()?)?,
                top: parse_u64(iter.next()?)?,
                bot: parse_u64(iter.next()?)?,
                left: parse_u64(iter.next()?)?,
                right: parse_u64(iter.next()?)?,
                rows: parse_u64(iter.next()?)?,
                cols: parse_u64(iter.next()?)?,
            })
        };
        inner().ok_or(GridScrollParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_scroll event")]
pub struct GridScrollParseError;

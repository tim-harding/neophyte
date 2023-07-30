use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

/// Set the current grid and cursor position
#[derive(Clone, Copy, Debug)]
pub struct GridCursorGoto {
    /// The current grid
    pub grid: u64,
    /// The cursor position row
    pub row: u64,
    /// The cursor position column
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

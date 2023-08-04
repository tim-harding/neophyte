use crate::util::{Parse, Values};
use rmpv::Value;

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

impl Parse for GridCursorGoto {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            row: iter.next()?,
            column: iter.next()?,
        })
    }
}

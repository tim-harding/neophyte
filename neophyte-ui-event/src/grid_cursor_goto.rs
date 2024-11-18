use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Set the current grid and cursor position
#[derive(Clone, Copy, Debug, Serialize)]
pub struct GridCursorGoto {
    /// The current grid
    pub grid: u32,
    /// The cursor position row
    pub row: u16,
    /// The cursor position column
    pub column: u16,
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

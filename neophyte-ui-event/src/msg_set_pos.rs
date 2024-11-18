use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Display messages on grid. The grid will be displayed at row on the default
/// grid (grid=1), covering the full column width. When ui-messages is active,
/// no message grid is used, and this event will not be sent.
#[derive(Debug, Clone, Serialize)]
pub struct MsgSetPos {
    /// The grid to display on the default grid
    pub grid: u32,
    /// The row of the default grid the messages will be displayed on
    pub row: u16,
    /// Whether the message area has been scrolled to cover other grids.
    pub scrolled: bool,
    /// The Builtin TUI draws a full line filled with sep_char and MsgSeparator
    /// highlight
    pub sep_char: String,
}

impl Parse for MsgSetPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            row: iter.next()?,
            scrolled: iter.next()?,
            sep_char: iter.next()?,
        })
    }
}

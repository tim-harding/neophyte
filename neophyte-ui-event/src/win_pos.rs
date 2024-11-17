use super::messagepack_ext_types::Window;
use crate::{Parse, Values};
use rmpv::Value;

/// Set the position and size of the outer grid size. If the window was
/// previously hidden, it should now be shown again.
#[derive(Debug, Clone)]
pub struct WinPos {
    /// The grid to update
    pub grid: u32,
    /// The window containing the grid
    pub win: Window,
    /// Top boundary
    pub start_row: u16,
    /// Lefthand boundary
    pub start_col: u16,
    /// New grid width
    pub width: u16,
    /// New grid height
    pub height: u16,
}

impl Parse for WinPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            start_row: iter.next()?,
            start_col: iter.next()?,
            width: iter.next()?,
            height: iter.next()?,
        })
    }
}

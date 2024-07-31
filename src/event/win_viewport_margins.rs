use super::messagepack_ext_types::Window;
use crate::util::{Parse, Values};
use rmpv::Value;

// TODO: Figure out how to use this event

/// Indicates the margins of a window grid which are _not_ part of the viewport
/// as indicated by the `win_viewport` event. This happens in the presence of
/// `winbar` and floating window borders.
#[derive(Debug, Clone)]
pub struct WinViewportMargins {
    /// The grid to update
    pub grid: u32,
    /// The window to update
    pub win: Window,
    /// The top margin
    pub top: u32,
    /// The bottom margin
    pub bottom: u32,
    /// The left margin
    pub left: u32,
    /// The right margin
    pub right: u32,
}

impl Parse for WinViewportMargins {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            top: iter.next()?,
            bottom: iter.next()?,
            left: iter.next()?,
            right: iter.next()?,
        })
    }
}

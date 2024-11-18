use super::messagepack_ext_types::Window;
use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Display or reconfigure a floating window.
#[derive(Debug, Clone, Serialize)]
pub struct WinFloatPos {
    /// The grid to display in the window
    pub grid: u32,
    /// The window to display
    pub win: Window,
    /// Which corner of the float to place at the anchor position
    pub anchor: Anchor,
    /// The grid to display the window over
    pub anchor_grid: u32,
    /// The row of the anchor grid at which to display the window
    pub anchor_row: f32,
    /// The column of the anchor grid at which to display the window
    pub anchor_col: f32,
    /// Whether the window can receive focus
    pub focusable: bool,
    // NOTE: Add documentation for z-index
    pub zindex: Option<u32>,
}

impl Parse for WinFloatPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            anchor: iter.next()?,
            anchor_grid: iter.next()?,
            anchor_row: iter.next()?,
            anchor_col: iter.next()?,
            focusable: iter.next()?,
            zindex: iter.next()?,
        })
    }
}

/// Which corner of the float to place at the anchor position
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Anchor {
    /// Northwest
    Nw,
    /// Northeast
    Ne,
    /// Southwest
    Sw,
    /// Southeast
    Se,
}

impl Parse for Anchor {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        match s.as_str() {
            "NW" => Some(Self::Nw),
            "NE" => Some(Self::Ne),
            "SW" => Some(Self::Sw),
            "SE" => Some(Self::Se),
            _ => None,
        }
    }
}

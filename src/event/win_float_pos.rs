use super::messagepack_ext_types::Window;
use crate::util::{Parse, Values};
use rmpv::Value;

/// Display or reconfigure a floating window.
#[derive(Debug, Clone)]
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
    // NOTE: There is an additional undocumented u32 parameter. Based on the
    // Neovide codebase, this is used to indicate stacking order. I choose to
    // ignore it unless the documentation is updated. Until then, I assume it is
    // not intended to be used.
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
        })
    }
}

/// Which corner of the float to place at the anchor position
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

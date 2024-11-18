use super::messagepack_ext_types::Window;
use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Display or reconfigure external window. The window should be displayed as a
/// separate top-level window in the desktop environment or something similar.
#[derive(Debug, Clone, Serialize)]
pub struct WinExternalPos {
    /// The grid to display in the window
    pub grid: u32,
    /// The window to display
    pub win: Window,
}

impl Parse for WinExternalPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
        })
    }
}

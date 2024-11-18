use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Resize a grid
#[derive(Debug, Clone, Copy, Serialize)]
pub struct GridResize {
    /// The grid to resize
    pub grid: u32,
    /// The new width
    pub width: u16,
    /// The new height
    pub height: u16,
}

impl Parse for GridResize {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            width: iter.next()?,
            height: iter.next()?,
        })
    }
}

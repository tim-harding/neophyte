use super::util::{Parse, Values};
use nvim_rs::Value;

/// Resize a grid
#[derive(Debug, Clone, Copy)]
pub struct GridResize {
    /// The grid to resize
    pub grid: u64,
    /// The new width
    pub width: u64,
    /// The new height
    pub height: u64,
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

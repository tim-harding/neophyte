use super::util::{parse_array, parse_u64};
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

impl GridResize {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter().map(parse_u64).flatten();
        Some(Self {
            grid: iter.next()?,
            width: iter.next()?,
            height: iter.next()?,
        })
    }
}

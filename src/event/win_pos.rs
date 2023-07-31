use super::{
    messagepack_ext_types::Window,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Copy, Clone)]
pub struct WinPos {
    pub grid: u64,
    pub win: Window,
    pub start_row: u64,
    pub start_col: u64,
    pub width: u64,
    pub height: u64,
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

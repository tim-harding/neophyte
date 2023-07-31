use super::{
    messagepack_ext_types::Window,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct WinExternalPos {
    pub grid: u64,
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

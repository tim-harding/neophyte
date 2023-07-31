use super::{
    messagepack_ext_types::Window,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct WinExtmark {
    pub grid: u64,
    pub win: Window,
    pub ns_id: u64,
    pub mark_id: u64,
    pub row: u64,
    pub col: u64,
}

impl Parse for WinExtmark {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            ns_id: iter.next()?,
            mark_id: iter.next()?,
            row: iter.next()?,
            col: iter.next()?,
        })
    }
}

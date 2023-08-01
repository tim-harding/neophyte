use super::{
    messagepack_ext_types::Window,
    util::{Parse, Values},
};
use nvim_rs::Value;

/// Updates the position of an extmark which is currently visible in a window.
#[derive(Debug, Clone)]
pub struct WinExtmark {
    /// The grid containing the extmark
    pub grid: u64,
    /// The window containing the extmark
    pub win: Window,
    /// Namespace ID
    pub ns_id: u64,
    /// Extmark ID
    pub mark_id: u64,
    /// Row the extmark is on
    pub row: u64,
    /// Column the extmark is on
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

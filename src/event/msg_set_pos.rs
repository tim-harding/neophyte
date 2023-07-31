use super::util::{Parse, Values};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct MsgSetPos {
    pub grid: u64,
    pub row: u64,
    pub scrolled: bool,
    pub sep_char: String,
}

impl Parse for MsgSetPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            row: iter.next()?,
            scrolled: iter.next()?,
            sep_char: iter.next()?,
        })
    }
}

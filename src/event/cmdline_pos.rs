use super::util::{Parse, Values};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct CmdlinePos {
    pub pos: u64,
    pub level: u64,
}

impl Parse for CmdlinePos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            pos: iter.next()?,
            level: iter.next()?,
        })
    }
}

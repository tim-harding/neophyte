use super::util::{Parse, Values};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct CmdlineSpecialChar {
    pub c: char,
    pub shift: bool,
    pub level: u64,
}

impl Parse for CmdlineSpecialChar {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            c: iter.next()?,
            shift: iter.next()?,
            level: iter.next()?,
        })
    }
}

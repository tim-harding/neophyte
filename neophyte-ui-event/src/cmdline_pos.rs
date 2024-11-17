use crate::{Parse, Values};
use rmpv::Value;

/// Change the cursor position in the cmdline.
#[derive(Debug, Clone, Copy)]
pub struct CmdlinePos {
    pub pos: u32,
    pub level: u32,
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

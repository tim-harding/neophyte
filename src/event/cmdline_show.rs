use super::{parse::Parse, types::MessageContent, util::Values};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct CmdlineShow {
    pub content: MessageContent,
    pub pos: u64,
    pub firstc: String,
    pub prompt: String,
    pub indent: u64,
    pub level: u64,
}

impl Parse for CmdlineShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            content: iter.next()?,
            pos: iter.next()?,
            firstc: iter.next()?,
            prompt: iter.next()?,
            indent: iter.next()?,
            level: iter.next()?,
        })
    }
}

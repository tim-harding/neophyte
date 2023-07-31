use super::{
    types::MessageContent,
    util::{parse_array, parse_string, parse_u64},
};
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

impl CmdlineShow {
    pub(super) fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            content: MessageContent::parse(iter.next()?)?,
            pos: parse_u64(iter.next()?)?,
            firstc: parse_string(iter.next()?)?,
            prompt: parse_string(iter.next()?)?,
            indent: parse_u64(iter.next()?)?,
            level: parse_u64(iter.next()?)?,
        })
    }
}

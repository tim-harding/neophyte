use super::{
    types::{Buffer, Tabpage},
    util::{parse_map, MaybeInto, Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct TablineUpdate {
    pub curtab: Tabpage,
    pub tabs: Vec<Tab>,
    pub curbuf: Buffer,
    pub buffers: Vec<BufferInfo>,
}

impl Parse for TablineUpdate {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            curtab: iter.next()?,
            tabs: iter.next()?,
            curbuf: iter.next()?,
            buffers: iter.next()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub tab: Tabpage,
    pub name: String,
}

impl Parse for Tab {
    fn parse(value: Value) -> Option<Self> {
        let map = parse_map(value)?;
        let mut tab = None;
        let mut name = None;
        for (k, v) in map {
            let k = String::parse(k)?;
            match k.as_str() {
                "tab" => tab = v.maybe_into(),
                "name" => name = v.maybe_into(),
                _ => return None,
            };
        }
        Some(Self {
            tab: tab?,
            name: name?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BufferInfo {
    pub buffer: Buffer,
    pub name: String,
}

impl Parse for BufferInfo {
    fn parse(value: Value) -> Option<Self> {
        let map = parse_map(value)?;
        let mut buffer = None;
        let mut name = None;
        for (k, v) in map {
            let k = String::parse(k)?;
            match k.as_str() {
                "buffer" => buffer = v.maybe_into(),
                "name" => name = v.maybe_into(),
                _ => return None,
            };
        }
        Some(Self {
            buffer: buffer?,
            name: name?,
        })
    }
}

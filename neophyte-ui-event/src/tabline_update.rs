use super::messagepack_ext_types::{Buffer, Tabpage};
use crate::{parse_map, MaybeInto, Parse, Values};
use rmpv::Value;

/// Tabline was updated.
#[derive(Debug, Clone)]
pub struct TablineUpdate {
    /// Current tabpage
    pub curtab: Tabpage,
    /// Tabpages
    pub tabs: Vec<TabpageInfo>,
    /// Current buffer
    pub curbuf: Buffer,
    /// Buffers
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
pub struct TabpageInfo {
    pub tab: Tabpage,
    pub name: String,
}

impl Parse for TabpageInfo {
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

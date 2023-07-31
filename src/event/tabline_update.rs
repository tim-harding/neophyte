use super::{
    types::{Buffer, Tabpage},
    util::{map_array, parse_array, parse_map, parse_string, Parse},
};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct TablineUpdate {
    pub curtab: Tabpage,
    pub tabs: Vec<Tab>,
    pub curbuf: Buffer,
    pub buffers: Vec<BufferInfo>,
}

impl TablineUpdate {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            curtab: Tabpage::parse(iter.next()?)?,
            tabs: map_array(iter.next()?, Tab::parse)?,
            curbuf: Buffer::parse(iter.next()?)?,
            buffers: map_array(iter.next()?, BufferInfo::parse)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Tab {
    pub tab: Tabpage,
    pub name: String,
}

impl Tab {
    pub fn parse(value: Value) -> Option<Self> {
        let map = parse_map(value)?;
        let mut tab = None;
        let mut name = None;
        for (k, v) in map {
            let k = parse_string(k)?;
            match k.as_str() {
                "tab" => tab = Tabpage::parse(v),
                "name" => name = parse_string(v),
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

impl BufferInfo {
    pub fn parse(value: Value) -> Option<Self> {
        let map = parse_map(value)?;
        let mut buffer = None;
        let mut name = None;
        for (k, v) in map {
            let k = parse_string(k)?;
            match k.as_str() {
                "buffer" => buffer = Buffer::parse(v),
                "name" => name = parse_string(v),
                _ => return None,
            };
        }
        Some(Self {
            buffer: buffer?,
            name: name?,
        })
    }
}

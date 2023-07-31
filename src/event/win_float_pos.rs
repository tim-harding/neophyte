use super::{
    messagepack_ext_types::Window,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct WinFloatPos {
    pub grid: u64,
    pub win: Window,
    pub anchor: Anchor,
    pub anchor_grid: u64,
    pub anchor_row: f64,
    pub anchor_col: f64,
    pub focusable: bool,
    // TODO: There is an additional undocumented u64 parameter. Investigate.
}

impl Parse for WinFloatPos {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            win: iter.next()?,
            anchor: iter.next()?,
            anchor_grid: iter.next()?,
            anchor_row: iter.next()?,
            anchor_col: iter.next()?,
            focusable: iter.next()?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    Nw,
    Ne,
    Sw,
    Se,
}

impl Parse for Anchor {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        match s.as_str() {
            "NW" => Some(Self::Nw),
            "NE" => Some(Self::Ne),
            "SW" => Some(Self::Sw),
            "SE" => Some(Self::Se),
            _ => None,
        }
    }
}

use super::{
    types::Window,
    util::{parse_string, ValueIter},
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

impl WinFloatPos {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = ValueIter::new(value)?;
        Some(Self {
            grid: iter.next_u64()?,
            win: Window::parse(iter.next()?)?,
            anchor: Anchor::parse(iter.next()?)?,
            anchor_grid: iter.next_u64()?,
            anchor_row: iter.next_f64()?,
            anchor_col: iter.next_f64()?,
            focusable: iter.next_bool()?,
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

impl Anchor {
    pub fn parse(value: Value) -> Option<Self> {
        let s = parse_string(value)?;
        match s.as_str() {
            "NW" => Some(Self::Nw),
            "NE" => Some(Self::Ne),
            "SW" => Some(Self::Sw),
            "SE" => Some(Self::Se),
            _ => None,
        }
    }
}

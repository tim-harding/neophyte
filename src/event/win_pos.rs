use super::{
    types::Window,
    util::{parse_array, parse_u64},
};
use nvim_rs::Value;

#[derive(Debug, Copy, Clone)]
pub struct WinPos {
    pub grid: u64,
    pub win: Window,
    pub start_row: u64,
    pub start_col: u64,
    pub width: u64,
    pub height: u64,
}

impl WinPos {
    pub(super) fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            grid: parse_u64(iter.next()?)?,
            win: Window::parse(iter.next()?)?,
            start_row: parse_u64(iter.next()?)?,
            start_col: parse_u64(iter.next()?)?,
            width: parse_u64(iter.next()?)?,
            height: parse_u64(iter.next()?)?,
        })
    }
}

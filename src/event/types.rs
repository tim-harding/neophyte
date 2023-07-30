use super::util::parse_ext;
use nvim_rs::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Window(u64);

impl Window {
    pub(super) fn parse(value: Value) -> Option<Self> {
        parse_ext(value, 1).map(vec_to_handle).map(Self)
    }
}

pub struct Tabpage(u64);

impl Tabpage {
    pub(super) fn parse(value: Value) -> Option<Self> {
        parse_ext(value, 2).map(vec_to_handle).map(Self)
    }
}

fn vec_to_handle(vec: Vec<u8>) -> u64 {
    assert!(vec.len() <= 8);
    let mut out = 0;
    for (i, v) in vec.into_iter().enumerate() {
        out |= (v as u64) << i * 8;
    }
    out
}

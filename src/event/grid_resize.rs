use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct GridResize {
    pub grid: u64,
    pub width: u64,
    pub height: u64,
}

impl TryFrom<Value> for GridResize {
    type Error = GridResizeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter().map(parse_u64).flatten();
            Some(Self {
                grid: iter.next()?,
                width: iter.next()?,
                height: iter.next()?,
            })
        };
        inner().ok_or(GridResizeParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_resize event")]
pub struct GridResizeParseError;

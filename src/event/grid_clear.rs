use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct GridClear {
    pub grids: Vec<u64>,
}

impl GridClear {
    pub fn new(grids: Vec<u64>) -> Self {
        Self { grids }
    }
}

impl GridClear {
    pub fn parse(value: Value) -> Option<Self> {
        let grids: Option<Vec<_>> = parse_array(value)?.into_iter().map(parse_u64).collect();
        Some(Self::new(grids?))
    }
}

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

impl TryFrom<Value> for GridClear {
    type Error = GridClearParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let grids: Option<Vec<_>> = parse_array(value)?.into_iter().map(parse_u64).collect();
            Some(Self::new(grids?))
        };
        inner().ok_or(GridClearParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_clear event")]
pub struct GridClearParseError;

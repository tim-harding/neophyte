use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct GridDestroy {
    pub grids: Vec<u64>,
}

impl GridDestroy {
    pub fn new(grids: Vec<u64>) -> Self {
        Self { grids }
    }
}

impl TryFrom<Value> for GridDestroy {
    type Error = GridDestroyParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let grids: Option<Vec<_>> = parse_array(value)?.into_iter().map(parse_u64).collect();
            Some(Self::new(grids?))
        };
        inner().ok_or(GridDestroyParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_destroy event")]
pub struct GridDestroyParseError;

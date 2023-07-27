use super::util::parse_u64;
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
        match value {
            Value::Array(array) => {
                let grids: Option<Vec<_>> = array.into_iter().map(parse_u64).collect();
                Ok(Self::new(grids.ok_or(GridDestroyParseError)?))
            }
            _ => Err(GridDestroyParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_destroy event")]
pub struct GridDestroyParseError;

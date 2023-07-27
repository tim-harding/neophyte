use super::util::parse_u64;
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
        match value {
            Value::Array(array) => {
                let grids: Option<Vec<_>> = array.into_iter().map(parse_u64).collect();
                Ok(Self::new(grids.ok_or(GridClearParseError)?))
            }
            _ => Err(GridClearParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_clear event")]
pub struct GridClearParseError;

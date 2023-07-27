use super::util::parse_u64;
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct GridClear {
    pub grid: u64,
}

impl GridClear {
    pub fn new(grid: u64) -> Self {
        Self { grid }
    }
}

impl TryFrom<Value> for GridClear {
    type Error = GridClearParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(Self::new(parse_u64(value).ok_or(GridClearParseError)?))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse set_title event")]
pub struct GridClearParseError;

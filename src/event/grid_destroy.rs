use super::util::parse_u64;
use nvim_rs::Value;

#[derive(Debug, Clone, Copy)]
pub struct GridDestroy {
    pub grid: u64,
}

impl GridDestroy {
    pub fn new(grid: u64) -> Self {
        Self { grid }
    }
}

impl TryFrom<Value> for GridDestroy {
    type Error = GridDestroyParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(Self::new(parse_u64(value).ok_or(GridDestroyParseError)?))
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_destroy event")]
pub struct GridDestroyParseError;

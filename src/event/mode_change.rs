use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct ModeChange {
    pub mode: String,
    pub mode_idx: u64,
}

impl TryFrom<Value> for ModeChange {
    type Error = ModeChangeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter();
            Some(Self {
                mode: parse_string(iter.next()?)?,
                mode_idx: parse_u64(iter.next()?)?,
            })
        };
        inner().ok_or(ModeChangeParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Error parsing mode_change event")]
pub struct ModeChangeParseError;

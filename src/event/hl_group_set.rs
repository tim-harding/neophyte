use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub struct HlGroupSet {
    pub hl_groups: Vec<HlGroup>,
}

impl TryFrom<IntoIter<Value>> for HlGroupSet {
    type Error = HlGroupSetParseError;

    fn try_from(values: IntoIter<Value>) -> Result<Self, Self::Error> {
        let hl_groups: Result<Vec<_>, _> = values.map(HlGroup::try_from).collect();
        Ok(Self {
            hl_groups: hl_groups?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HlGroup {
    pub name: String,
    pub hl_id: u64,
}

impl TryFrom<Value> for HlGroup {
    type Error = HlGroupSetParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter();
            Some(Self {
                name: parse_string(iter.next()?)?,
                hl_id: parse_u64(iter.next()?)?,
            })
        };
        inner().ok_or(HlGroupSetParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse hl_group_set event")]
pub struct HlGroupSetParseError;

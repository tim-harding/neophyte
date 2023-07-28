use super::util::{parse_array, parse_string};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct SetIcon(Vec<String>);

impl TryFrom<Value> for SetIcon {
    type Error = SetIconParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = || -> Option<Self> {
            let titles: Option<Vec<_>> =
                parse_array(value)?.into_iter().map(parse_string).collect();
            Some(Self(titles?))
        };
        inner().ok_or(SetIconParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse set_title event")]
pub struct SetIconParseError;

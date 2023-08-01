use super::util::{parse_first_element, parse_maybe_u64, Parse};
use nvim_rs::Value;

/// Set the minimized window title
#[derive(Debug, Clone)]
pub struct PopupmenuSelect {
    pub selected: Option<u64>,
}

impl Parse for PopupmenuSelect {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            selected: parse_maybe_u64(parse_first_element(value)?)?,
        })
    }
}

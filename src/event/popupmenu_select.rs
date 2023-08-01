use super::util::{parse_first_element, parse_maybe_u64, Parse};
use nvim_rs::Value;

/// Select an item in the current popupmenu.
#[derive(Debug, Clone)]
pub struct PopupmenuSelect {
    /// The item to select, or None if no item is selected
    pub selected: Option<u64>,
}

impl Parse for PopupmenuSelect {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            selected: parse_maybe_u64(parse_first_element(value)?)?,
        })
    }
}

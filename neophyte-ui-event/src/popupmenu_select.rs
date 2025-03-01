use crate::{Parse, parse_first_element, parse_maybe_u32};
use rmpv::Value;
use serde::Serialize;

/// Select an item in the current popupmenu.
#[derive(Debug, Clone, Serialize)]
pub struct PopupmenuSelect {
    /// The item to select, or None if no item is selected
    pub selected: Option<u32>,
}

impl Parse for PopupmenuSelect {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            selected: parse_maybe_u32(parse_first_element(value)?)?,
        })
    }
}

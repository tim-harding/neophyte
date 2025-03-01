use crate::{MaybeInto, Parse, parse_first_element};
use rmpv::Value;
use serde::Serialize;

/// Stop displaying the window. The window can be shown again later.
#[derive(Debug, Clone, Serialize)]
pub struct WinHide {
    pub grid: u32,
}

impl Parse for WinHide {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

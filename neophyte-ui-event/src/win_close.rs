use crate::{MaybeInto, Parse, parse_first_element};
use rmpv::Value;
use serde::Serialize;

/// Stop displaying the window. The window can be shown again later.
#[derive(Debug, Clone, Serialize)]
pub struct WinClose {
    pub grid: u32,
}

impl Parse for WinClose {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

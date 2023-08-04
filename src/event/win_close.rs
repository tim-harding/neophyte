use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Stop displaying the window. The window can be shown again later.
#[derive(Debug, Clone)]
pub struct WinClose {
    pub grid: u64,
}

impl Parse for WinClose {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

use super::util::{parse_first_element, MaybeInto, Parse};
use nvim_rs::Value;

/// Stop displaying the window. The window can be shown again later.
#[derive(Debug, Clone)]
pub struct WinHide {
    pub grid: u64,
}

impl Parse for WinHide {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

use super::util::{parse_first_element, MaybeInto, Parse};
use nvim_rs::Value;

/// Set the minimized window title
#[derive(Debug, Clone)]
pub struct GridClear {
    pub grid: u64,
}

impl Parse for GridClear {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

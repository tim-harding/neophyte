use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Clear a grid
#[derive(Debug, Clone)]
pub struct GridClear {
    pub grid: u32,
}

impl Parse for GridClear {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

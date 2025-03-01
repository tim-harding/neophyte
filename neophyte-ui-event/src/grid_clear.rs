use crate::{MaybeInto, Parse, parse_first_element};
use rmpv::Value;
use serde::Serialize;

/// Clear a grid
#[derive(Debug, Clone, Serialize)]
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

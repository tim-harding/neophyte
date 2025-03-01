use crate::{MaybeInto, Parse, parse_first_element};
use rmpv::Value;
use serde::Serialize;

/// Grid will not be used anymore and the UI can free any data associated with it.
#[derive(Debug, Clone, Serialize)]
pub struct GridDestroy {
    pub grid: u32,
}

impl Parse for GridDestroy {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

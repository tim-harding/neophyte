use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Grid will not be used anymore and the UI can free any data associated with it.
#[derive(Debug, Clone)]
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

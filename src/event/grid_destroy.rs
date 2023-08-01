use super::util::{parse_first_element, MaybeInto, Parse};
use nvim_rs::Value;

/// Grid will not be used anymore and the UI can free any data associated with it.
#[derive(Debug, Clone)]
pub struct GridDestroy {
    pub grid: u64,
}

impl Parse for GridDestroy {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            grid: parse_first_element(value)?.maybe_into()?,
        })
    }
}

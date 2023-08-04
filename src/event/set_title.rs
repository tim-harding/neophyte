use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Set the global window title
#[derive(Debug, Clone)]
pub struct SetTitle {
    pub title: String,
}

impl Parse for SetTitle {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            title: parse_first_element(value)?.maybe_into()?,
        })
    }
}

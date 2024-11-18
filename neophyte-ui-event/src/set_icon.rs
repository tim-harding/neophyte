use crate::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;
use serde::Serialize;

/// Set the global minimized window title
#[derive(Debug, Clone, Serialize)]
pub struct SetIcon {
    pub icon: String,
}

impl Parse for SetIcon {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            icon: parse_first_element(value)?.maybe_into()?,
        })
    }
}

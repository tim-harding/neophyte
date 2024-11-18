use super::message_content::Content;
use crate::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;
use serde::Serialize;

/// Show a block of text to the current command line, for example if the user
/// defines a function interactively.
#[derive(Debug, Clone, Serialize)]
pub struct CmdlineBlockShow {
    pub lines: Vec<Content>,
}

impl Parse for CmdlineBlockShow {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            lines: parse_first_element(value)?.maybe_into()?,
        })
    }
}

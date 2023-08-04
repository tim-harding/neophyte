use super::message_content::Content;
use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Append a line at the end of the currently shown block.
#[derive(Debug, Clone)]
pub struct CmdlineBlockAppend {
    pub line: Content,
}

impl Parse for CmdlineBlockAppend {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            line: parse_first_element(value)?.maybe_into()?,
        })
    }
}

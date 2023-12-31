use super::message_content::Content;
use crate::util::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Shows 'showcmd'. This event is sent with empty content to hide the last
/// message.
#[derive(Debug, Clone)]
pub struct MsgShowcmd {
    pub content: Content,
}

impl Parse for MsgShowcmd {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            content: parse_first_element(value)?.maybe_into()?,
        })
    }
}

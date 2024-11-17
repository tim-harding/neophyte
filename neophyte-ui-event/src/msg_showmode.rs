use super::message_content::Content;
use crate::{parse_first_element, MaybeInto, Parse};
use rmpv::Value;

/// Shows 'showmode' and recording messages. This event is sent with empty
/// content to hide the last message.
#[derive(Debug, Clone)]
pub struct MsgShowmode {
    pub content: Content,
}

impl Parse for MsgShowmode {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            content: parse_first_element(value)?.maybe_into()?,
        })
    }
}

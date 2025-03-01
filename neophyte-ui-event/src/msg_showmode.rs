use super::message_content::Content;
use crate::{MaybeInto, Parse, parse_first_element};
use rmpv::Value;
use serde::Serialize;

/// Shows 'showmode' and recording messages. This event is sent with empty
/// content to hide the last message.
#[derive(Debug, Clone, Serialize)]
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

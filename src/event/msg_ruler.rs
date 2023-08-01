use super::{
    message_content::Content,
    util::{parse_first_element, MaybeInto, Parse},
};
use nvim_rs::Value;

/// Used to display 'ruler' when there is no space for the ruler in a
/// statusline. This event is sent with empty content to hide the last message.
#[derive(Debug, Clone)]
pub struct MsgRuler {
    pub content: Content,
}

impl Parse for MsgRuler {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            content: parse_first_element(value)?.maybe_into()?,
        })
    }
}

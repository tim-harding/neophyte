use super::{
    message_content::Content,
    util::{parse_first_element, MaybeInto, Parse},
};
use nvim_rs::Value;

/// Set the minimized window title
#[derive(Debug, Clone)]
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

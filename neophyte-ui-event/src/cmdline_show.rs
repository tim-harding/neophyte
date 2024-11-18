use super::message_content::Content;
use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Triggered when the cmdline is displayed or changed.
#[derive(Debug, Clone, Serialize)]
pub struct CmdlineShow {
    /// The full content that should be displayed in the cmdline.
    pub content: Content,
    /// The position of the cursor that in the cmdline.
    pub pos: u32,
    /// Text displayed in front of the command line, such as :/?
    pub firstc: String,
    /// A prompt displayed in front of the command line as provided by input()
    pub prompt: String,
    /// How many spaces the content should be indented
    pub indent: u32,
    /// Distinguishes different command lines active at the same time, for
    /// example after <c-r>= in a prompt
    pub level: u32,
}

impl Parse for CmdlineShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            content: iter.next()?,
            pos: iter.next()?,
            firstc: iter.next()?,
            prompt: iter.next()?,
            indent: iter.next()?,
            level: iter.next()?,
        })
    }
}

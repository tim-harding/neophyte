use crate::{MaybeInto, Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// A text sequence with highlighting.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Content {
    /// Chunks to be concatenated without extra spacing
    pub chunks: Vec<ContentChunk>,
}

impl Parse for Content {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            chunks: value.maybe_into()?,
        })
    }
}

/// A highlighted portion of text
#[derive(Debug, Clone, Serialize)]
pub struct ContentChunk {
    /// The ID of the highlight to apply
    pub attr_id: u32,
    /// The text
    pub text_chunk: String,
}

impl Parse for ContentChunk {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            attr_id: iter.next()?,
            text_chunk: maybe_escape_newlines(iter.next()?),
        })
    }
}

fn maybe_escape_newlines(s: String) -> String {
    if s.contains("\\n") {
        s.replace("\\n", "\n")
    } else {
        s
    }
}

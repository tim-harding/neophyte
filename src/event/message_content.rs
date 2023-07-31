use super::util::{MaybeInto, Parse, Values};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct Content {
    pub chunks: Vec<ContentChunk>,
}

impl Parse for Content {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            chunks: value.maybe_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ContentChunk {
    pub attr_id: u64,
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

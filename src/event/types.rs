use super::util::{parse_array, parse_ext, parse_string, parse_u64, Parse};
use nvim_rs::Value;

fn vec_to_handle(vec: Vec<u8>) -> u64 {
    assert!(vec.len() <= 8);
    let mut out = 0;
    for (i, v) in vec.into_iter().enumerate() {
        out |= (v as u64) << i * 8;
    }
    out
}

macro_rules! msgpack_ext {
    ($x:ident, $n:expr) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $x(u64);

        impl Parse for $x {
            fn parse(value: Value) -> Option<Self> {
                parse_ext(value, $n).map(vec_to_handle).map(Self)
            }
        }
    };
}

msgpack_ext!(Buffer, 0);
msgpack_ext!(Window, 1);
msgpack_ext!(Tabpage, 2);

#[derive(Debug, Clone)]
pub struct MessageContent {
    pub chunks: Vec<MessageContentChunk>,
}

impl MessageContent {
    pub fn parse(value: Value) -> Option<Self> {
        let chunks: Option<Vec<_>> = parse_array(value)?
            .into_iter()
            .map(MessageContentChunk::parse)
            .collect();
        Some(Self { chunks: chunks? })
    }
}

#[derive(Debug, Clone)]
pub struct MessageContentChunk {
    pub attr_id: u64,
    pub text_chunk: String,
}

impl MessageContentChunk {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            attr_id: parse_u64(iter.next()?)?,
            text_chunk: parse_string(iter.next()?).map(maybe_escape_newlines)?,
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

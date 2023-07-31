use super::{
    parse::Parse,
    util::{parse_ext, MaybeInto, Values},
};
use nvim_rs::Value;

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

impl Parse for MessageContent {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            chunks: value.maybe_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MessageContentChunk {
    pub attr_id: u64,
    pub text_chunk: String,
}

impl Parse for MessageContentChunk {
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

fn vec_to_handle(vec: Vec<u8>) -> u64 {
    assert!(vec.len() <= 8);
    let mut out = 0;
    for (i, v) in vec.into_iter().enumerate() {
        out |= (v as u64) << i * 8;
    }
    out
}

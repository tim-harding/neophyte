use super::util::{parse_ext, Parse};
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

fn vec_to_handle(vec: Vec<u8>) -> u64 {
    assert!(vec.len() <= 8);
    let mut out = 0;
    for (i, v) in vec.into_iter().enumerate() {
        out |= (v as u64) << i * 8;
    }
    out
}

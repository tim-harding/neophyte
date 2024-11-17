use crate::Parse;
use rmpv::Value;

macro_rules! msgpack_ext {
    ($x:ident, $doc:meta) => {
        #[derive(Debug, Clone)]
        #[$doc]
        pub struct $x(#[allow(unused)] Value);

        impl Parse for $x {
            fn parse(value: Value) -> Option<Self> {
                Some(Self(value))
            }
        }
    };
}

msgpack_ext!(Buffer, doc = "A handle to a Neovim buffer");
msgpack_ext!(Window, doc = "A handle to a Neovim window");
msgpack_ext!(Tabpage, doc = "A handle to a Neovim tabpage");

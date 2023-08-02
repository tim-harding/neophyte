use super::util::Parse;
use crate::nvim::{Nvim, Writer};
use nvim_rs::Value;

macro_rules! msgpack_ext {
    ($x:ident, $doc:meta) => {
        #[derive(Debug, Clone)]
        #[$doc]
        pub struct $x(Value);

        impl Parse for $x {
            fn parse(value: Value) -> Option<Self> {
                Some(Self(value))
            }
        }

        impl $x {
            #[allow(unused)]
            pub fn into_nvim_rs(self, nvim: Nvim) -> nvim_rs::$x<Writer> {
                nvim_rs::$x::new(self.0, nvim)
            }
        }
    };
}

msgpack_ext!(Buffer, doc = "A handle to a Neovim buffer");
msgpack_ext!(Window, doc = "A handle to a Neovim window");
msgpack_ext!(Tabpage, doc = "A handle to a Neovim tabpage");

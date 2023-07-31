use super::{
    message_content::Content,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct MsgShow {
    pub kind: Kind,
    pub content: Content,
    pub replace_last: bool,
}

impl Parse for MsgShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            kind: iter.next()?,
            content: iter.next()?,
            replace_last: iter.next()?,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    /// Unknown
    Unknown,
    /// Confirm dialog from :confirm
    Confirm,
    /// Substitute confirm dialog, :s_c
    ConfirmSub,
    /// Error message
    Emsg,
    /// From :echo
    Echo,
    /// From :echomsg
    Echomsg,
    /// From :echoerr
    Echoerr,
    /// Error in Lua code
    LuaError,
    /// Error response from rpcrequest()
    RpcError,
    /// press-enter prompt after multiple messages
    ReturnPrompt,
    /// Quickfix navigation message
    Quickfix,
    /// Search count message from shortmess S flag
    SearchCount,
    /// Warning, e.g. "search hit BOTTOM"
    Wmsg,
}

impl Parse for Kind {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        Some(match s.as_str() {
            "Confirm" => Self::Confirm,
            "ConfirmSub" => Self::ConfirmSub,
            "Emsg" => Self::Emsg,
            "Echo" => Self::Echo,
            "Echomsg" => Self::Echomsg,
            "Echoerr" => Self::Echoerr,
            "LuaError" => Self::LuaError,
            "RpcError" => Self::RpcError,
            "ReturnPrompt" => Self::ReturnPrompt,
            "Quickfix" => Self::Quickfix,
            "SearchCount" => Self::SearchCount,
            "Wmsg" => Self::Wmsg,
            _ => Self::Unknown,
        })
    }
}

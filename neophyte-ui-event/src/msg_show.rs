use super::message_content::Content;
use crate::{Parse, Values};
use rmpv::Value;

/// Display a message to the user.
#[derive(Debug, Clone)]
pub struct MsgShow {
    /// The kind of message
    pub kind: Kind,
    /// The text to display
    pub content: Content,
    /// Whether to replace the previous message
    pub replace_last: ReplaceLast,
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

/// Whether to replace the previous message
#[derive(Debug, Clone)]
pub enum ReplaceLast {
    /// Replace the message in the most-recent msg_show call, but any other
    /// visible message should still remain.
    Replace,
    /// Display the message together with all previous messages that are still
    /// visible.
    Keep,
}

impl Parse for ReplaceLast {
    fn parse(value: Value) -> Option<Self> {
        Some(bool::parse(value)?.into())
    }
}

impl From<bool> for ReplaceLast {
    fn from(value: bool) -> Self {
        if value {
            Self::Replace
        } else {
            Self::Keep
        }
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

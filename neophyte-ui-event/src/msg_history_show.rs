use super::{message_content::Content, msg_show::Kind};
use crate::{Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Sent when :messages command is invoked
#[derive(Debug, Clone, Serialize)]
pub struct MsgHistoryShow {
    pub entries: Vec<MsgHistoryEntry>,
}

impl Parse for MsgHistoryShow {
    fn parse(value: Value) -> Option<Self> {
        Some(Self {
            entries: Values::new(value)?.next()?,
        })
    }
}

/// A message history item in the msg_history_show event
#[derive(Debug, Clone, Serialize)]
pub struct MsgHistoryEntry {
    /// The message kind
    pub kind: Kind,
    /// The message content
    pub content: Content,
}

impl Parse for MsgHistoryEntry {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            kind: iter.next()?,
            content: iter.next()?,
        })
    }
}

use super::{
    message_content::Content,
    msg_show::Kind,
    util::{Parse, Values},
};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct MsgHistoryShow {
    pub kind: Kind,
    pub content: Content,
}

impl Parse for MsgHistoryShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            kind: iter.next()?,
            content: iter.next()?,
        })
    }
}

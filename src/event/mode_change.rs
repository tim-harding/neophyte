use super::{parse::Parse, util::Values};
use nvim_rs::Value;

/// Editor mode changed.
#[derive(Debug, Clone)]
pub struct ModeChange {
    /// The current mode
    pub mode: String,
    /// An index into the array emitted in the mode_info_set event
    pub mode_idx: u64,
}

impl Parse for ModeChange {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            mode: iter.next()?,
            mode_idx: iter.next()?,
        })
    }
}

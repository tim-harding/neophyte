use super::util::ValueIter;
use nvim_rs::Value;

/// Editor mode changed.
#[derive(Debug, Clone)]
pub struct ModeChange {
    /// The current mode
    pub mode: String,
    /// An index into the array emitted in the mode_info_set event
    pub mode_idx: u64,
}

impl ModeChange {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = ValueIter::new(value)?;
        Some(Self {
            mode: iter.next()?,
            mode_idx: iter.next()?,
        })
    }
}

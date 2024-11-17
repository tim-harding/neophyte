use super::{Parse, Values};
use rmpv::Value;

/// Set the position and size of the outer grid size. If the window was
/// previously hidden, it should now be shown again.
#[derive(Debug, Clone)]
pub struct Chdir {
    /// The current directory to change to
    pub path: String,
}

impl Parse for Chdir {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self { path: iter.next()? })
    }
}

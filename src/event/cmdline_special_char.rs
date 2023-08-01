use super::util::{Parse, Values};
use nvim_rs::Value;

/// Display a special char in the cmdline at the cursor position. This is
/// typically used to indicate a pending state, such as after <C-V>.
#[derive(Debug, Clone, Copy)]
pub struct CmdlineSpecialChar {
    /// The special character.
    pub c: char,
    /// Whether the cursor should be shifted. Otherwise, overwrite the character
    /// at the cursor.
    pub shift: bool,
    /// Distinguishes different command lines active at the same time.
    pub level: u64,
}

impl Parse for CmdlineSpecialChar {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            c: iter.next()?,
            shift: iter.next()?,
            level: iter.next()?,
        })
    }
}

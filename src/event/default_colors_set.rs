use super::util::{Parse, ValueIter};
use nvim_rs::Value;

/// Sets the default foreground, background, and special colors.
#[derive(Debug, Copy, Clone)]
pub struct DefaultColorsSet {
    /// Foreground in RGB
    pub rgb_fg: u64,
    /// Background in RGB
    pub rgb_bg: u64,
    /// Special color in RGB
    pub rgb_sp: u64,
    /// Foreground for 256-color terminals
    pub cterm_fg: u64,
    /// Background for 256-color terminals
    pub cterm_bg: u64,
}

impl Parse for DefaultColorsSet {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = ValueIter::new(value)?;
        Some(Self {
            rgb_fg: iter.next()?,
            rgb_bg: iter.next()?,
            rgb_sp: iter.next()?,
            cterm_fg: iter.next()?,
            cterm_bg: iter.next()?,
        })
    }
}

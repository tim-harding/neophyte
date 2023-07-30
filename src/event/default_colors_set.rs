use super::util::{parse_array, parse_u64};
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

impl DefaultColorsSet {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter().map(parse_u64);
        let mut next = || iter.next().flatten();
        Some(Self {
            rgb_fg: next()?,
            rgb_bg: next()?,
            rgb_sp: next()?,
            cterm_fg: next()?,
            cterm_bg: next()?,
        })
    }
}

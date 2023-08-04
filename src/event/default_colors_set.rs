use crate::util::{parse_maybe_u64, Parse, Values};
use rmpv::Value;

/// Sets the default foreground, background, and special colors.
#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultColorsSet {
    /// Foreground in RGB
    pub rgb_fg: Option<u64>,
    /// Background in RGB
    pub rgb_bg: Option<u64>,
    /// Special color in RGB
    pub rgb_sp: Option<u64>,
    /// Foreground for 256-color terminals
    pub cterm_fg: Option<u64>,
    /// Background for 256-color terminals
    pub cterm_bg: Option<u64>,
}

impl Parse for DefaultColorsSet {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            rgb_fg: parse_maybe_u64(iter.next()?)?,
            rgb_bg: parse_maybe_u64(iter.next()?)?,
            rgb_sp: parse_maybe_u64(iter.next()?)?,
            cterm_fg: parse_maybe_u64(iter.next()?)?,
            cterm_bg: parse_maybe_u64(iter.next()?)?,
        })
    }
}

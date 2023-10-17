use super::rgb::Rgb;
use crate::util::{parse_maybe_u64, Parse, Values};
use rmpv::Value;

/// Sets the default foreground, background, and special colors.
#[derive(Debug, Copy, Clone, Default)]
pub struct DefaultColorsSet {
    /// Foreground in RGB
    pub rgb_fg: Option<Rgb>,
    /// Background in RGB
    pub rgb_bg: Option<Rgb>,
    /// Special color in RGB
    pub rgb_sp: Option<Rgb>,
    /// Foreground for 256-color terminals
    pub cterm_fg: Option<u8>,
    /// Background for 256-color terminals
    pub cterm_bg: Option<u8>,
}

impl Parse for DefaultColorsSet {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            rgb_fg: parse_maybe_u64(iter.next()?)?.map(Rgb::from),
            rgb_bg: parse_maybe_u64(iter.next()?)?.map(Rgb::from),
            rgb_sp: parse_maybe_u64(iter.next()?)?.map(Rgb::from),
            cterm_fg: parse_maybe_u64(iter.next()?)?.map(|v| v as u8),
            cterm_bg: parse_maybe_u64(iter.next()?)?.map(|v| v as u8),
        })
    }
}

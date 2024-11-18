use super::rgb::Rgb;
use crate::{parse_maybe_u32, Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Sets the default foreground, background, and special colors.
#[derive(Debug, Copy, Clone, Default, Serialize)]
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
            rgb_fg: parse_maybe_u32(iter.next()?)?.map(Rgb::from),
            rgb_bg: parse_maybe_u32(iter.next()?)?.map(Rgb::from),
            rgb_sp: parse_maybe_u32(iter.next()?)?.map(Rgb::from),
            cterm_fg: parse_maybe_u32(iter.next()?)?.and_then(|v| u8::try_from(v).ok()),
            cterm_bg: parse_maybe_u32(iter.next()?)?.and_then(|v| u8::try_from(v).ok()),
        })
    }
}

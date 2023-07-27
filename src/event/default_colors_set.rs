use super::util::{parse_array, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Copy, Clone)]
pub struct DefaultColorsSet {
    pub rgb_fg: u64,
    pub rgb_bg: u64,
    pub rgb_sp: u64,
    pub cterm_fg: u64,
    pub cterm_bg: u64,
}

impl TryFrom<Value> for DefaultColorsSet {
    type Error = DefaultColorsSetParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter().map(parse_u64);
            let mut next = || iter.next().flatten();
            Some(Self {
                rgb_fg: next()?,
                rgb_bg: next()?,
                rgb_sp: next()?,
                cterm_fg: next()?,
                cterm_bg: next()?,
            })
        };
        inner().ok_or(DefaultColorsSetParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse default_colors_set event")]
pub struct DefaultColorsSetParseError;

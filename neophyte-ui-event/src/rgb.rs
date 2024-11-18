use crate::Parse;
use rmpv::Value;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self::new((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }
}

impl Parse for Rgb {
    fn parse(value: Value) -> Option<Self> {
        Some(Self::from(u32::parse(value)?))
    }
}

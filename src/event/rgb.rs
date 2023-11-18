use crate::util::{srgb, Parse};
use rmpv::Value;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn into_linear(self) -> [f32; 4] {
        [linear(self.r), linear(self.g), linear(self.b), 1.0]
    }
}

fn linear(c: u8) -> f32 {
    srgb(c).powf(2.2f32.recip())
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

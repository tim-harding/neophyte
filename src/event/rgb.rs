use rmpv::Value;

use crate::util::{srgb, Parse};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rgb([u8; 3]);

impl Rgb {
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }

    pub const fn r(&self) -> u8 {
        self.0[0]
    }

    pub const fn g(&self) -> u8 {
        self.0[1]
    }

    pub const fn b(&self) -> u8 {
        self.0[2]
    }

    pub const fn into_array(self) -> [u8; 3] {
        self.0
    }

    pub fn into_linear(self) -> [f32; 4] {
        [linear(self.r()), linear(self.g()), linear(self.b()), 1.0]
    }
}

fn linear(c: u8) -> f32 {
    srgb(c).powf(2.2f32.recip())
}

impl From<u64> for Rgb {
    fn from(value: u64) -> Self {
        Self::new((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }
}

impl From<Rgb> for [u8; 3] {
    fn from(value: Rgb) -> Self {
        value.0
    }
}

impl Parse for Rgb {
    fn parse(value: Value) -> Option<Self> {
        Some(Self::from(u64::parse(value)?))
    }
}

use std::fmt::{self, Debug, Formatter};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackedChar(u32);

impl PackedChar {
    const SURROGATE_LOW: u32 = 0xD800;
    const SURROGATE_HIGH: u32 = 0xDFFF;
    const SURROGATE_MASK: u32 = Self::SURROGATE_LOW & Self::SURROGATE_HIGH;
    const LEADING: u32 = (char::MAX as u32).leading_zeros(); // 11
    const LEADING_MASK: u32 = !(u32::MAX >> Self::LEADING);
    const TRAILING: u32 = Self::SURROGATE_LOW.trailing_zeros(); // 11
    const TRAILING_MASK: u32 = !(u32::MAX << Self::TRAILING);
    const CHAR_MASK: u32 = !Self::LEADING_MASK;
    const MAX_U22_LEADING: u32 = U22::MAX.leading_zeros();

    pub const fn from_char(c: char) -> Self {
        Self(c as u32)
    }

    pub const fn from_u22(u22: U22) -> Self {
        let n = u22.as_u32();
        let leading = (n << Self::MAX_U22_LEADING) & Self::LEADING_MASK;
        let trailing = n & Self::TRAILING_MASK;
        Self(leading | trailing | Self::SURROGATE_MASK)
    }

    pub fn contents(self) -> PackedCharContents {
        let c = self.0 & Self::CHAR_MASK;
        if c < Self::SURROGATE_LOW || c > Self::SURROGATE_HIGH {
            // TODO: Make this function const when from_u32_unchecked as const
            // is stablized.
            PackedCharContents::Char(unsafe { char::from_u32_unchecked(c) })
        } else {
            let i = self.0 & !Self::SURROGATE_MASK;
            let trailing = i & Self::TRAILING_MASK;
            let leading = i & Self::LEADING_MASK;
            PackedCharContents::U22(unsafe {
                U22::from_u32_unchecked(trailing | (leading >> Self::MAX_U22_LEADING))
            })
        }
    }
}

impl Debug for PackedChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.contents())
    }
}

impl From<char> for PackedChar {
    fn from(c: char) -> Self {
        Self::from_char(c)
    }
}

impl From<U22> for PackedChar {
    fn from(u22: U22) -> Self {
        Self::from_u22(u22)
    }
}

impl TryFrom<u32> for PackedChar {
    type Error = U22FromU32Error;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        let u22 = U22::from_u32(n)?;
        Ok(Self::from_u22(u22))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U22(u32);

impl U22 {
    pub const MAX: u32 = !(u32::MAX << 22);

    pub const fn from_u32(n: u32) -> Result<Self, U22FromU32Error> {
        if n > Self::MAX {
            Err(U22FromU32Error(n))
        } else {
            Ok(Self(n))
        }
    }

    pub const unsafe fn from_u32_unchecked(n: u32) -> Self {
        Self(n)
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for U22 {
    type Error = U22FromU32Error;

    fn try_from(n: u32) -> Result<Self, Self::Error> {
        Self::from_u32(n)
    }
}

impl From<U22> for u32 {
    fn from(u22: U22) -> Self {
        u22.0
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("{0} exceeds U22::MAX ({})", U22::MAX)]
pub struct U22FromU32Error(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackedCharContents {
    Char(char),
    U22(U22),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_back_chars() {
        let test_chars = ['\0', '\u{D7FF}', '\u{E000}', '\u{10FFFF}', 'a', '1', '🫠'];
        for c in test_chars {
            let packed = PackedChar::from_char(c);
            assert_eq!(packed.contents(), PackedCharContents::Char(c));
        }
    }

    #[test]
    fn gets_back_indices() {
        let test_indices = [U22::MAX, 0x3FFFFFu32, 0, 69, 420, 0b1010101010101010101010];
        for i in test_indices {
            let packed = PackedChar::try_from(i).unwrap();
            assert_eq!(
                packed.contents(),
                PackedCharContents::U22(U22::try_from(i).unwrap())
            );
        }
    }

    #[test]
    fn fails_out_of_bounds_indices() {
        let test_indices = [U22::MAX + 1, 0b10101010101010101010101010101010];
        for i in test_indices {
            let packed = PackedChar::try_from(i);
            assert_eq!(packed, Err(U22FromU32Error(i)));
        }
    }
}

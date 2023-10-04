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
    pub const MAX_INDEX: u32 = !(u32::MAX << (Self::LEADING + Self::TRAILING));
    const MAX_INDEX_LEADING: u32 = Self::MAX_INDEX.leading_zeros();

    pub const fn from_char(c: char) -> Self {
        Self(c as u32)
    }

    pub const fn from_index(i: u32) -> Result<Self, PackedCharIndexError> {
        if i > Self::MAX_INDEX {
            return Err(PackedCharIndexError(i));
        }
        let leading = (i << Self::MAX_INDEX_LEADING) & Self::LEADING_MASK;
        let trailing = i & Self::TRAILING_MASK;
        Ok(Self(leading | trailing | Self::SURROGATE_MASK))
    }

    pub fn contents(self) -> PackedCharContents {
        let c = self.0 & Self::CHAR_MASK;
        match char::from_u32(c) {
            Some(c) => PackedCharContents::Char(c),
            None => {
                let i = self.0 & !Self::SURROGATE_MASK;
                let trailing = i & Self::TRAILING_MASK;
                let leading = i & Self::LEADING_MASK;
                PackedCharContents::Index(trailing | (leading >> Self::MAX_INDEX_LEADING))
            }
        }
    }
}

impl Debug for PackedChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.contents())
    }
}

#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("The PackedChar index {0} exceeded {}", PackedChar::MAX_INDEX)]
pub struct PackedCharIndexError(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PackedCharContents {
    Char(char),
    Index(u32),
}

impl From<char> for PackedChar {
    fn from(c: char) -> Self {
        Self::from_char(c)
    }
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
        let test_indices = [
            PackedChar::MAX_INDEX,
            0x3FFFFFu32,
            0,
            69,
            420,
            0b1010101010101010101010,
        ];
        for i in test_indices {
            let packed = PackedChar::from_index(i).unwrap();
            assert_eq!(packed.contents(), PackedCharContents::Index(i));
        }
    }

    #[test]
    fn fails_out_of_bounds_indices() {
        let test_indices = [0x1000000u32, 0b10101010101010101010101010101010];
        for i in test_indices {
            let packed = PackedChar::from_index(i);
            assert_eq!(packed, Err(PackedCharIndexError(i)));
        }
    }
}

use crate::neovim::button::Button;

#[derive(PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord, Debug)]
pub struct Buttons(u8);

#[rustfmt::skip]
impl Buttons {
    const LEFT:   u8 = 0b001;
    const RIGHT:  u8 = 0b010;
    const MIDDLE: u8 = 0b100;
}

impl Buttons {
    pub const fn with_left(self, value: bool) -> Self {
        Self(self.0 | (Self::LEFT * value as u8))
    }

    pub const fn with_right(self, value: bool) -> Self {
        Self(self.0 | (Self::RIGHT * value as u8))
    }

    pub const fn with_middle(self, value: bool) -> Self {
        Self(self.0 | (Self::MIDDLE * value as u8))
    }

    pub const fn left(self) -> bool {
        self.0 & Self::LEFT > 0
    }

    pub const fn right(self) -> bool {
        self.0 & Self::RIGHT > 0
    }

    pub const fn middle(self) -> bool {
        self.0 & Self::MIDDLE > 0
    }

    pub fn first(&self) -> Option<Button> {
        if self.left() {
            Some(Button::Left)
        } else if self.right() {
            Some(Button::Right)
        } else if self.middle() {
            Some(Button::Middle)
        } else {
            None
        }
    }
}

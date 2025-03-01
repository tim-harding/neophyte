use crate::neovim::button::Button;

#[derive(PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord, Debug)]
pub struct Buttons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl Buttons {
    pub fn first(&self) -> Option<Button> {
        if self.left {
            Some(Button::Left)
        } else if self.right {
            Some(Button::Right)
        } else if self.middle {
            Some(Button::Middle)
        } else {
            None
        }
    }
}

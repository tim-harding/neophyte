use rmpv::Value;
use winit::event::MouseButton;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Wheel,
    Move,
}

impl From<Button> for &str {
    fn from(button: Button) -> Self {
        match button {
            Button::Left => "left",
            Button::Right => "right",
            Button::Middle => "middle",
            Button::Wheel => "wheel",
            Button::Move => "move",
        }
    }
}

impl From<Button> for Value {
    fn from(button: Button) -> Self {
        let s: &str = button.into();
        s.to_string().into()
    }
}

impl TryFrom<MouseButton> for Button {
    type Error = ButtonFromWinitError;

    fn try_from(button: MouseButton) -> Result<Self, Self::Error> {
        match button {
            MouseButton::Left => Ok(Self::Left),
            MouseButton::Right => Ok(Self::Right),
            MouseButton::Middle => Ok(Self::Middle),
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => {
                Err(ButtonFromWinitError)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No Neovim button for the given Winit mouse button")]
pub struct ButtonFromWinitError;

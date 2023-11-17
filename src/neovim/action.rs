use rmpv::Value;
use winit::event::ElementState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ButtonPress,
    ButtonDrag,
    ButtonRelease,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

impl From<Action> for &str {
    fn from(action: Action) -> Self {
        match action {
            Action::ButtonPress => "press",
            Action::ButtonDrag => "drag",
            Action::ButtonRelease => "release",
            Action::WheelUp => "up",
            Action::WheelDown => "down",
            Action::WheelLeft => "left",
            Action::WheelRight => "right",
        }
    }
}

impl From<Action> for Value {
    fn from(action: Action) -> Self {
        let s: &str = action.into();
        s.to_string().into()
    }
}

impl From<ElementState> for Action {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::ButtonPress,
            ElementState::Released => Self::ButtonRelease,
        }
    }
}

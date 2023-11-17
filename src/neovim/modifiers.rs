use rmpv::Value;
use winit::keyboard::ModifiersState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Modifiers(u8);

#[rustfmt::skip]
impl Modifiers {
    const CTRL:  u8 = 0b0001;
    const SHIFT: u8 = 0b0010;
    const ALT:   u8 = 0b0100;
    const LOGO:  u8 = 0b1000;
}

impl Modifiers {
    pub const fn new() -> Self {
        Self(0)
    }

    pub fn with_ctrl(self, value: bool) -> Self {
        Self(self.0 | (Self::CTRL * value as u8))
    }

    pub fn ctrl(self) -> bool {
        self.0 & Self::CTRL > 0
    }

    pub fn with_shift(self, value: bool) -> Self {
        Self(self.0 | (Self::SHIFT * value as u8))
    }

    pub fn shift(self) -> bool {
        self.0 & Self::SHIFT > 0
    }

    pub fn with_alt(self, value: bool) -> Self {
        Self(self.0 | (Self::ALT * value as u8))
    }

    pub fn alt(self) -> bool {
        self.0 & Self::ALT > 0
    }

    pub fn with_logo(self, value: bool) -> Self {
        Self(self.0 | (Self::LOGO * value as u8))
    }

    pub fn logo(self) -> bool {
        self.0 & Self::LOGO > 0
    }
}

impl From<Modifiers> for String {
    fn from(mods: Modifiers) -> Self {
        let ctrl = if mods.ctrl() { "C" } else { "" };
        let shift = if mods.shift() { "S" } else { "" };
        let alt = if mods.alt() { "A" } else { "" };
        format!("{ctrl}{shift}{alt}")
    }
}

impl From<Modifiers> for Value {
    fn from(modifiers: Modifiers) -> Self {
        let s: String = modifiers.into();
        s.into()
    }
}

impl From<ModifiersState> for Modifiers {
    fn from(state: ModifiersState) -> Self {
        Self::new()
            .with_ctrl(state.control_key())
            .with_shift(state.shift_key())
            .with_alt(state.alt_key())
            .with_logo(state.super_key())
    }
}

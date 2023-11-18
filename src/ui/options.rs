impl From<String> for GuiFont {
    fn from(option: String) -> Self {
        let mut out = GuiFont::default();
        let mut state = ParseState::Normal;
        let mut current = String::new();
        for c in option.chars() {
            match state {
                ParseState::Normal => {
                    if c.is_whitespace() && current.is_empty() {
                        continue;
                    }

                    match c {
                        '\\' => state = ParseState::Escape,
                        ',' => {
                            out.fonts.push(current);
                            current = String::default();
                            state = ParseState::Normal;
                        }
                        '_' => current.push(' '),
                        ':' => state = ParseState::OptionStart,
                        _ => current.push(c),
                    }
                }

                ParseState::Escape => current.push(c),

                ParseState::OptionStart => {
                    state = match c {
                        'w' | 'W' => ParseState::OptionSize(0, SizeKind::Width),
                        'h' | 'H' => ParseState::OptionSize(0, SizeKind::Height),
                        _ => ParseState::OptionUnknown,
                    };
                }

                ParseState::OptionSize(size, kind) => match c {
                    '0'..='9' => {
                        let size = size * 10 + c as u32 - '0' as u32;
                        state = ParseState::OptionSize(size, kind);
                    }
                    ',' => {
                        out.fonts.push(current);
                        out.size = FontSize::new(size as f32, kind);
                        current = String::default();
                        state = ParseState::Normal;
                    }
                    ':' => {
                        out.size = FontSize::new(size as f32, kind);
                        state = ParseState::OptionStart;
                    }
                    _ => {
                        log::warn!("Bad font height option");
                        break;
                    }
                },

                ParseState::OptionUnknown => match c {
                    ',' => {
                        out.fonts.push(current);
                        current = String::default();
                        state = ParseState::Normal;
                    }
                    ':' => state = ParseState::OptionStart,
                    _ => {}
                },
            }
        }

        if !current.is_empty() {
            out.fonts.push(current);
        }

        out
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct GuiFont {
    pub fonts: Vec<String>,
    pub size: FontSize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    Width(f32),
    Height(f32),
}

impl FontSize {
    fn new(size: f32, kind: SizeKind) -> Self {
        match kind {
            SizeKind::Width => Self::Width(size),
            SizeKind::Height => Self::Height(size),
        }
    }
}

impl Default for FontSize {
    fn default() -> Self {
        Self::Height(18.)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    /// Appending chars as normal
    Normal,
    /// Found a backslash escape sequence
    Escape,
    /// Found a :
    OptionStart,
    /// Found a :h or :w
    OptionSize(u32, SizeKind),
    /// An unknown option is being specified
    OptionUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeKind {
    Width,
    Height,
}

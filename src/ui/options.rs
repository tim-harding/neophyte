use crate::event::{
    option_set::{Ambiwidth, Showtabline},
    OptionSet,
};

#[derive(Debug, Default, Clone)]
pub struct Options {
    /// Enable shaping
    pub arabicshape: bool,
    /// ,hat to do with characters with East Asian Width Class Ambiguous
    pub ambiwidth: Ambiwidth,
    /// Emoji characters are considered to be full width
    pub emoji: bool,
    /// Fonts to use with fallbacks
    pub guifont: FontsSetting,
    /// Fonts to use for double-width characters with fallbacks
    pub guifontwide: FontsSetting,
    /// Number of pixel lines inserted between characters
    pub linespace: u64,
    /// Window focus follows mouse pointer
    pub mousefocus: bool,
    /// mouse move events are available for mapping
    pub mousemoveevent: bool,
    /// Enables pseudo-transparency for the popup-menu. Valid values are in the
    /// range of 0 for fully opaque popupmenu (disabled) to 100 for fully
    /// transparent background
    pub pumblend: u64,
    /// When the line with tab page labels will be displayed
    pub showtabline: Showtabline,
}

impl Options {
    pub fn event(&mut self, event: OptionSet) {
        match event {
            OptionSet::Arabicshape(value) => self.arabicshape = value,
            OptionSet::Ambiwidth(value) => self.ambiwidth = value,
            OptionSet::Emoji(value) => self.emoji = value,
            OptionSet::Guifont(value) => self.guifont = fonts_from_option(value),
            OptionSet::Guifontwide(value) => self.guifontwide = fonts_from_option(value),
            OptionSet::Linespace(value) => self.linespace = value,
            OptionSet::Mousefocus(value) => self.mousefocus = value,
            OptionSet::Mousemoveevent(value) => self.mousemoveevent = value,
            OptionSet::Pumblend(value) => self.pumblend = value,
            OptionSet::Showtabline(value) => self.showtabline = value,
            _ => {}
        }
    }
}

fn fonts_from_option(option: String) -> FontsSetting {
    let mut out = FontsSetting::default();
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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct FontsSetting {
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

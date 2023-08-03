use crate::event::{
    option_set::{Ambiwidth, Showtabline},
    OptionSet,
};

#[derive(Debug, Default, Clone)]
pub struct Options {
    /// Enable shaping
    arabicshape: bool,
    /// ,hat to do with characters with East Asian Width Class Ambiguous
    ambiwidth: Ambiwidth,
    /// Emoji characters are considered to be full width
    emoji: bool,
    /// Fonts to use with fallbacks
    guifont: Vec<String>,
    /// Fonts to use for double-width characters with fallbacks
    guifontwide: Vec<String>,
    /// Number of pixel lines inserted between characters
    linespace: u64,
    /// Window focus follows mouse pointer
    mousefocus: bool,
    /// mouse move events are available for mapping
    mousemoveevent: bool,
    /// Enables pseudo-transparency for the popup-menu. Valid values are in the
    /// range of 0 for fully opaque popupmenu (disabled) to 100 for fully
    /// transparent background
    pumblend: u64,
    /// When the line with tab page labels will be displayed
    showtabline: Showtabline,
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

fn fonts_from_option(option: String) -> Vec<String> {
    let mut start = 0;
    let mut leading_whitespace = true;
    let mut found_escape = false;
    let mut out = vec![];
    for (i, char) in option.chars().enumerate() {
        if char.is_whitespace() && leading_whitespace {
            start = i;
        }
        leading_whitespace = false;
        match char {
            '\\' => found_escape = true,
            ',' => {
                if found_escape {
                    out.push(option.chars().skip(start).take(i - start).collect());
                    start = i;
                    leading_whitespace = true;
                }
                found_escape = false;
            }
            _ => found_escape = false,
        }
        if char == '\\' {
            found_escape = true;
        }
        if char == ',' && found_escape {}
    }

    if out.is_empty() {
        vec![option]
    } else {
        out.push(option.chars().skip(start).collect());
        out
    }
}

use super::rgb::Rgb;
use crate::{maybe_field, parse_map, Parse, Values};
use rmpv::Value;
use std::fmt::{self, Debug, Formatter};

/// Add a highlight with id to the highlight table
#[derive(Debug, Clone, Default)]
pub struct HlAttrDefine {
    pub id: u32,
    /// Highlights in RGB format
    pub rgb_attr: Attributes,
    /// Highlights in terminal 256-color codes
    pub cterm_attr: Attributes,
    /// A semantic description of the highlights active in a cell. Ordered by
    /// priority from low to high.
    pub info: Vec<Info>,
}

impl Parse for HlAttrDefine {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            id: iter.next()?,
            rgb_attr: iter.next()?,
            cterm_attr: iter.next()?,
            info: iter.next()?,
        })
    }
}

// NOTE: Ignoring the altfont attribute. Not sure what this type should be or
// what it is used for.

/// Attributes of a highlight attribute definition. Colors may be given in RGB
/// or terminal 256-color.
#[derive(Clone, Copy)]
pub struct Attributes {
    /// foreground color.
    pub foreground: Option<Rgb>,
    /// background color.
    pub background: Option<Rgb>,
    /// color to use for various underlines, when present.
    pub special: Option<Rgb>,
    packed: u16,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
            special: None,
            packed: 100 << Self::BLEND_TRAILING,
        }
    }
}

#[rustfmt::skip]
impl Attributes {
    const REVERSE:       u16 = 0b0000000000000001;
    const ITALIC:        u16 = 0b0000000000000010;
    const BOLD:          u16 = 0b0000000000000100;
    const STRIKETHROUGH: u16 = 0b0000000000001000;
    const UNDERLINE:     u16 = 0b0000000000010000;
    const UNDERCURL:     u16 = 0b0000000000100000;
    const UNDERDOUBLE:   u16 = 0b0000000001000000;
    const UNDERDOTTED:   u16 = 0b0000000010000000;
    const UNDERDASHED:   u16 = 0b0000000100000000;
    const BLEND_MASK:    u16 = 0b1111111000000000;
}

impl Attributes {
    const BLEND_TRAILING: u32 = Self::BLEND_MASK.trailing_zeros();

    /// reverse video. Foreground and background colors are switched.
    pub fn reverse(&self) -> bool {
        self.packed & Self::REVERSE > 0
    }

    /// italic text.
    pub fn italic(&self) -> bool {
        self.packed & Self::ITALIC > 0
    }

    /// bold text.
    pub fn bold(&self) -> bool {
        self.packed & Self::BOLD > 0
    }

    /// struckthrough text.
    pub fn strikethrough(&self) -> bool {
        self.packed & Self::STRIKETHROUGH > 0
    }

    /// underlined text. The line has special color.
    pub fn underline(&self) -> bool {
        self.packed & Self::UNDERLINE > 0
    }

    /// undercurled text. The curl has special color.
    pub fn undercurl(&self) -> bool {
        self.packed & Self::UNDERCURL > 0
    }

    /// double underlined text. The lines have special color.
    pub fn underdouble(&self) -> bool {
        self.packed & Self::UNDERDOUBLE > 0
    }

    /// underdotted text. The dots have special color.
    pub fn underdotted(&self) -> bool {
        self.packed & Self::UNDERDOTTED > 0
    }

    /// underdashed text. The dashes have special color.
    pub fn underdashed(&self) -> bool {
        self.packed & Self::UNDERDASHED > 0
    }

    /// Blend level (0-100). Could be used by UIs to support blending floating
    /// windows to the background or to signal a transparent cursor
    pub fn blend(&self) -> f32 {
        let percentage = (self.packed & Self::BLEND_MASK >> Self::BLEND_TRAILING) as u8;
        f32::from(100 - percentage) / 100.
    }

    fn maybe_set(&mut self, value: Value, mask: u16) -> Option<()> {
        let b = bool::parse(value)?;
        self.packed |= u16::from(b) * mask;
        Some(())
    }
}

impl Parse for Attributes {
    fn parse(value: Value) -> Option<Self> {
        let mut out = Self::default();
        for (k, v) in parse_map(value)? {
            let k = String::parse(k)?;
            match k.as_str() {
                "foreground" => out.foreground = Some(Parse::parse(v)?),
                "background" => out.background = Some(Parse::parse(v)?),
                "special" => out.special = Some(Parse::parse(v)?),
                "reverse" => out.maybe_set(v, Self::REVERSE)?,
                "italic" => out.maybe_set(v, Self::ITALIC)?,
                "bold" => out.maybe_set(v, Self::BOLD)?,
                "strikethrough" => out.maybe_set(v, Self::STRIKETHROUGH)?,
                "underline" => out.maybe_set(v, Self::UNDERLINE)?,
                "undercurl" => out.maybe_set(v, Self::UNDERCURL)?,
                "underdouble" => out.maybe_set(v, Self::UNDERDOUBLE)?,
                "underdotted" => out.maybe_set(v, Self::UNDERDOTTED)?,
                "underdashed" => out.maybe_set(v, Self::UNDERDASHED)?,
                "blend" => {
                    let blend = u16::parse(v)?;
                    out.packed &= !Self::BLEND_MASK;
                    out.packed |= blend << Self::BLEND_TRAILING;
                }
                _ => {} // Ignore undocumented attributes
            }
        }
        Some(out)
    }
}

impl Debug for Attributes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Attributes");
        maybe_field(&mut s, "foreground", self.foreground);
        maybe_field(&mut s, "background", self.background);
        maybe_field(&mut s, "special", self.special);
        s.field("reverse", &self.reverse());
        s.field("italic", &self.italic());
        s.field("bold", &self.bold());
        s.field("strikethrough", &self.strikethrough());
        s.field("underline", &self.underline());
        s.field("undercurl", &self.undercurl());
        s.field("underdouble", &self.underdouble());
        s.field("underdotted", &self.underdotted());
        s.field("underdashed", &self.underdashed());
        s.field("blend", &self.blend());
        s.finish()
    }
}

/// A semantic description of the highlights active in a cell. Activated by the
/// ext_hlstate extension.
#[derive(Debug, Clone)]
pub struct Info {
    pub kind: Kind,
    /// Highlight name from highlight-groups. Only for "ui" kind.
    pub ui_name: Option<String>,
    /// Name of the
    pub hi_name: Option<String>,
    /// Unique numeric id representing this item.
    pub id: Option<u32>,
}

impl Parse for Info {
    fn parse(value: Value) -> Option<Self> {
        let mut kind = None;
        let mut ui_name = None;
        let mut hi_name = None;
        let mut id = None;
        let map = parse_map(value)?;
        for (k, v) in map {
            let k = String::parse(k)?;
            match k.as_str() {
                "kind" => kind = Some(Parse::parse(v)?),
                "ui_name" => ui_name = Some(Parse::parse(v)?),
                "hi_name" => hi_name = Some(Parse::parse(v)?),
                "id" => id = Some(Parse::parse(v)?),
                _ => return None,
            }
        }
        Some(Self {
            kind: kind?,
            ui_name,
            hi_name,
            id,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    /// Builtin UI highlight.
    Ui,
    /// Highlight applied to a buffer by a syntax declaration or other
    /// runtime/plugin functionality such as nvim_buf_add_highlight()
    Syntax,
    /// Highlight from a process running in a terminal-emulator. Contains no
    /// further semantic information.
    Terminal,
}

impl Parse for Kind {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        match s.as_str() {
            "ui" => Some(Self::Ui),
            "syntax" => Some(Self::Syntax),
            "terminal" => Some(Self::Terminal),
            _ => None,
        }
    }
}

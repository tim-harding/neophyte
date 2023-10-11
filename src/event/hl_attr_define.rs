use crate::util::{maybe_field, maybe_other_field, parse_map, srgb, Parse, Values};
use rmpv::Value;
use std::fmt::{self, Debug, Formatter};

/// Add a highlight with id to the highlight table
#[derive(Debug, Clone, Default)]
pub struct HlAttrDefine {
    pub id: u64,
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

/// Attributes of a highlight attribute definition. Colors may be given in RGB
/// or terminal 256-color.
#[derive(Clone, Default)]
pub struct Attributes {
    /// foreground color.
    pub foreground: Option<Rgb>,
    /// background color.
    pub background: Option<Rgb>,
    /// color to use for various underlines, when present.
    pub special: Option<Rgb>,
    /// reverse video. Foreground and background colors are switched.
    pub reverse: Option<bool>,
    /// italic text.
    pub italic: Option<bool>,
    /// bold text.
    pub bold: Option<bool>,
    /// struckthrough text.
    pub strikethrough: Option<bool>,
    /// underlined text. The line has special color.
    pub underline: Option<bool>,
    /// undercurled text. The curl has special color.
    pub undercurl: Option<bool>,
    /// double underlined text. The lines have special color.
    pub underdouble: Option<bool>,
    /// underdotted text. The dots have special color.
    pub underdotted: Option<bool>,
    /// underdashed text. The dashes have special color.
    pub underdashed: Option<bool>,
    /// alternative font.
    pub altfont: Option<bool>,
    /// Blend level (0-100). Could be used by UIs to support blending floating
    /// windows to the background or to signal a transparent cursor
    pub blend: Option<u64>,
    /// Options not enumerated in the UI documentation
    pub other: Vec<(String, Value)>,
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
                "reverse" => out.reverse = Some(Parse::parse(v)?),
                "italic" => out.italic = Some(Parse::parse(v)?),
                "bold" => out.bold = Some(Parse::parse(v)?),
                "strikethrough" => out.strikethrough = Some(Parse::parse(v)?),
                "underline" => out.underline = Some(Parse::parse(v)?),
                "undercurl" => out.undercurl = Some(Parse::parse(v)?),
                "underdouble" => out.underdouble = Some(Parse::parse(v)?),
                "underdotted" => out.underdotted = Some(Parse::parse(v)?),
                "underdashed" => out.underdashed = Some(Parse::parse(v)?),
                "altfont" => out.altfont = Some(Parse::parse(v)?),
                "blend" => out.blend = Some(Parse::parse(v)?),
                _ => out.other.push((k, v)),
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
        maybe_field(&mut s, "reverse", self.reverse);
        maybe_field(&mut s, "italic", self.italic);
        maybe_field(&mut s, "bold", self.bold);
        maybe_field(&mut s, "strikethrough", self.strikethrough);
        maybe_field(&mut s, "underline", self.underline);
        maybe_field(&mut s, "undercurl", self.undercurl);
        maybe_field(&mut s, "underdouble", self.underdouble);
        maybe_field(&mut s, "underdotted", self.underdotted);
        maybe_field(&mut s, "underdashed", self.underdashed);
        maybe_field(&mut s, "altfont", self.altfont);
        maybe_field(&mut s, "blend", self.blend);
        maybe_other_field(&mut s, &self.other);
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
    pub id: Option<u64>,
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Rgb([u8; 3]);

impl Rgb {
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }

    pub const fn r(&self) -> u8 {
        self.0[0]
    }

    pub const fn g(&self) -> u8 {
        self.0[1]
    }

    pub const fn b(&self) -> u8 {
        self.0[2]
    }

    pub const fn into_array(self) -> [u8; 3] {
        self.0
    }

    pub fn into_linear(self) -> [f32; 4] {
        [linear(self.r()), linear(self.g()), linear(self.b()), 1.0]
    }
}

fn linear(c: u8) -> f32 {
    srgb(c).powf(2.2f32.recip())
}

impl From<u64> for Rgb {
    fn from(value: u64) -> Self {
        Self::new((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }
}

impl From<Rgb> for [u8; 3] {
    fn from(value: Rgb) -> Self {
        value.0
    }
}

impl Parse for Rgb {
    fn parse(value: Value) -> Option<Self> {
        Some(Self::from(u64::parse(value)?))
    }
}

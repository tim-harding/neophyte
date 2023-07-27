use super::util::{parse_array, parse_bool, parse_map, parse_string, parse_u64};
use nvim_rs::Value;
use std::vec::IntoIter;

#[derive(Debug, Clone)]
pub struct HlAttrDefine {
    pub hl_attrs: Vec<HlAttr>,
}

impl TryFrom<IntoIter<Value>> for HlAttrDefine {
    type Error = HlAttrDefineParseError;

    fn try_from(values: IntoIter<Value>) -> Result<Self, Self::Error> {
        let hl_attrs: Result<Vec<_>, _> = values.into_iter().map(HlAttr::try_from).collect();
        Ok(Self {
            hl_attrs: hl_attrs?,
        })
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum HlAttrDefineParseError {
    #[error("Error parsing HlAttr")]
    HlAttr,
    #[error("Error parsing Attributes")]
    Attributes,
    #[error("Error parsing Info")]
    Info,
    #[error("Error parsing Kind")]
    Kind,
}

/// Add a highlight with id to the highlight table
#[derive(Debug, Clone)]
pub struct HlAttr {
    pub id: u64,
    /// Highlights in RGB format
    pub rgb_attr: Attributes,
    /// Highlights in terminal 256-color codes
    pub cterm_attr: Attributes,
    /// A semantic description of the highlights active in a cell
    pub info: Info,
}

impl TryFrom<Value> for HlAttr {
    type Error = HlAttrDefineParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        use HlAttrDefineParseError::HlAttr as HlAttrError;
        let mut iter = parse_array(value).ok_or(HlAttrError)?.into_iter();
        let mut next = move || iter.next().ok_or(HlAttrError);
        Ok(Self {
            id: parse_u64(next()?).ok_or(HlAttrError)?,
            rgb_attr: Attributes::try_from(next()?)?,
            cterm_attr: Attributes::try_from(next()?)?,
            info: Info::try_from(next()?)?,
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Attributes {
    /// foreground color.
    pub foreground: Option<u64>,
    /// background color.
    pub background: Option<u64>,
    /// color to use for various underlines, when present.
    pub special: Option<u64>,
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
    /// Blend level (0-100). Could be used by UIs to support blending floating windows to the
    /// background or to signal a transparent cursor
    pub blend: Option<u64>,
}

impl TryFrom<Value> for Attributes {
    type Error = HlAttrDefineParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut out = Self::default();
            for pair in parse_map(value)? {
                let (k, v) = pair;
                let k = parse_string(k)?;
                match k.as_str() {
                    "foreground" => out.foreground = Some(parse_u64(v)?),
                    "background" => out.background = Some(parse_u64(v)?),
                    "special" => out.special = Some(parse_u64(v)?),
                    "reverse" => out.reverse = Some(parse_bool(v)?),
                    "italic" => out.italic = Some(parse_bool(v)?),
                    "bold" => out.bold = Some(parse_bool(v)?),
                    "strikethrough" => out.strikethrough = Some(parse_bool(v)?),
                    "underline" => out.underline = Some(parse_bool(v)?),
                    "undercurl" => out.undercurl = Some(parse_bool(v)?),
                    "underdouble" => out.underdouble = Some(parse_bool(v)?),
                    "underdotted" => out.underdotted = Some(parse_bool(v)?),
                    "underdashed" => out.underdashed = Some(parse_bool(v)?),
                    "altfont" => out.altfont = Some(parse_bool(v)?),
                    "blend" => out.blend = Some(parse_u64(v)?),
                    _ => eprintln!("Unknown highlight attribute: {k}"),
                }
            }
            Some(out)
        };
        inner().ok_or(HlAttrDefineParseError::Attributes)
    }
}

/// A semantic description of the highlights active in a cell. Activated by the ext_hlstate
/// extension.
#[derive(Debug, Clone)]
pub struct Info {
    pub kind: Kind,
    /// Highlight name from highlight-groups. Only for "ui" kind.
    pub ui_name: String,
    /// Name of the
    pub hi_name: String,
    /// Highlight group where the used attributes are defined.
    pub r#final: String,
    /// Unique numeric id representing this item.
    pub id: String,
}

impl TryFrom<Value> for Info {
    type Error = HlAttrDefineParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        // TODO: Implement
        Ok(Self {
            kind: Kind::Ui,
            ui_name: Default::default(),
            hi_name: Default::default(),
            r#final: Default::default(),
            id: Default::default(),
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Kind {
    /// Builtin UI highlight.
    Ui,
    /// Highlight applied to a buffer by a syntax declaration or other runtime/plugin functionality
    /// such as nvim_buf_add_highlight()
    Syntax,
    /// highlight from a process running in a terminal-emulator. Contains no further semantic
    /// information.
    Terminal,
}

impl TryFrom<Value> for Kind {
    type Error = HlAttrDefineParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let s = parse_string(value)?;
            match s.as_str() {
                "ui" => Some(Self::Ui),
                "syntax" => Some(Self::Syntax),
                "terminal" => Some(Self::Terminal),
                _ => None,
            }
        };
        inner().ok_or(HlAttrDefineParseError::Kind)
    }
}

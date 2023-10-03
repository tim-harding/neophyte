use crate::util::{maybe_field, maybe_other_field, parse_map, Parse, Values};
use rmpv::Value;
use std::fmt::{self, Debug, Formatter};

/// Information about editor modes. These will be used by the mode_change event.
#[derive(Debug, Clone)]
pub struct ModeInfoSet {
    /// Whether the UI should set the cursor style
    pub cursor_style_enabled: bool,
    /// Information about different modes
    pub mode_info: Vec<ModeInfo>,
}

impl Parse for ModeInfoSet {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            cursor_style_enabled: iter.next()?,
            mode_info: iter.next()?,
        })
    }
}

/// Information about a mode
#[derive(Clone, Default)]
pub struct ModeInfo {
    /// The mode name
    pub name: Option<String>,
    /// Mode code name, such as 'n' or 'v'.
    pub short_name: Option<String>,
    /// The cursor shape
    pub cursor_shape: Option<CursorShape>,
    /// Cell percentage occupied by the cursor
    pub cell_percentage: Option<u64>,
    /// Milliseconds delay before the cursor starts blinking
    pub blinkwait: Option<u64>,
    /// Milliseconds that the cursor is shown when blinking
    pub blinkon: Option<u64>,
    /// Milliseconds that the cursor is hidden when blinking
    pub blinkoff: Option<u64>,
    /// Cursor attribute ID defined by an hl_attr_define event
    pub attr_id: Option<u64>,
    /// Cursor attribute ID when langmap is active
    pub attr_id_lm: Option<u64>,
    /// Options not enumerated in the UI documentation
    pub other: Vec<(String, Value)>,
}

impl Parse for ModeInfo {
    fn parse(value: Value) -> Option<Self> {
        let mut out = Self::default();
        let value = parse_map(value)?;
        for (k, v) in value {
            let k = String::parse(k)?;
            match k.as_str() {
                "cursor_shape" => out.cursor_shape = Some(Parse::parse(v)?),
                "cell_percentage" => out.cell_percentage = Some(Parse::parse(v)?),
                "blinkwait" => out.blinkwait = Some(Parse::parse(v)?),
                "blinkon" => out.blinkon = Some(Parse::parse(v)?),
                "blinkoff" => out.blinkoff = Some(Parse::parse(v)?),
                "attr_id" => out.attr_id = Some(Parse::parse(v)?),
                "attr_id_lm" => out.attr_id_lm = Some(Parse::parse(v)?),
                "short_name" => out.short_name = Some(Parse::parse(v)?),
                "name" => out.name = Some(Parse::parse(v)?),
                _ => out.other.push((k, v)),
            }
        }
        Some(out)
    }
}

impl Debug for ModeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ModeInfo");
        maybe_field(&mut s, "cursor_shape", self.cursor_shape);
        maybe_field(&mut s, "cell_percentage", self.cell_percentage);
        maybe_field(&mut s, "blinkwait", self.blinkwait);
        maybe_field(&mut s, "blinkon", self.blinkon);
        maybe_field(&mut s, "blinkoff", self.blinkoff);
        maybe_field(&mut s, "attr_id", self.attr_id);
        maybe_field(&mut s, "attr_id_lm", self.attr_id_lm);
        maybe_field(&mut s, "short_name", self.short_name.as_ref());
        maybe_field(&mut s, "name", self.name.as_ref());
        maybe_other_field(&mut s, &self.other);
        s.finish()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Horizontal,
    Vertical,
}

impl Parse for CursorShape {
    fn parse(value: Value) -> Option<Self> {
        match String::parse(value)?.as_str() {
            "block" => Some(Self::Block),
            "horizontal" => Some(Self::Horizontal),
            "vertical" => Some(Self::Vertical),
            _ => None,
        }
    }
}

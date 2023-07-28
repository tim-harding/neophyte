use super::util::{maybe_field, parse_array, parse_bool, parse_map, parse_string, parse_u64};
use nvim_rs::Value;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct ModeInfoSet {
    pub cursor_style_enabled: bool,
    pub mode_info: Vec<ModeInfo>,
}

impl TryFrom<Value> for ModeInfoSet {
    type Error = ModeInfoSetParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut iter = parse_array(value)?.into_iter();
            Some(Self {
                cursor_style_enabled: parse_bool(iter.next()?)?,
                mode_info: parse_array(iter.next()?)?
                    .into_iter()
                    .map(ModeInfo::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?,
            })
        };
        inner().ok_or(ModeInfoSetParseError)
    }
}

#[derive(Clone, Default)]
pub struct ModeInfo {
    pub cursor_shape: Option<CursorShape>,
    pub cell_percentage: Option<u64>,
    pub blinkwait: Option<u64>,
    pub blinkon: Option<u64>,
    pub blinkoff: Option<u64>,
    pub attr_id: Option<u64>,
    pub attr_id_lm: Option<u64>,
    pub short_name: Option<String>,
    pub name: Option<String>,
}

impl TryFrom<Value> for ModeInfo {
    type Error = ModeInfoSetParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            let mut out = Self::default();
            let value = parse_map(value)?;
            for (k, v) in value {
                let k = parse_string(k)?;
                match k.as_str() {
                    "cursor_shape" => out.cursor_shape = Some(CursorShape::try_from(v).ok()?),
                    "cell_percentage" => out.cell_percentage = Some(parse_u64(v)?),
                    "blinkwait" => out.blinkwait = Some(parse_u64(v)?),
                    "blinkon" => out.blinkon = Some(parse_u64(v)?),
                    "blinkoff" => out.blinkoff = Some(parse_u64(v)?),
                    "attr_id" => out.attr_id = Some(parse_u64(v)?),
                    "attr_id_lm" => out.attr_id_lm = Some(parse_u64(v)?),
                    "short_name" => out.short_name = Some(parse_string(v)?),
                    "name" => out.name = Some(parse_string(v)?),
                    _ => eprintln!("Unknown mode_info_set key: {k}"),
                }
            }
            Some(out)
        };
        inner().ok_or(ModeInfoSetParseError)
    }
}

impl Debug for ModeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        s.finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CursorShape {
    Block,
    Horizontal,
    Vertical,
}

impl TryFrom<Value> for CursorShape {
    type Error = ModeInfoSetParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let inner = move || -> Option<Self> {
            match parse_string(value)?.as_str() {
                "block" => Some(Self::Block),
                "horizontal" => Some(Self::Horizontal),
                "vertical" => Some(Self::Vertical),
                _ => None,
            }
        };
        inner().ok_or(ModeInfoSetParseError)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse mode_info_set event")]
pub struct ModeInfoSetParseError;

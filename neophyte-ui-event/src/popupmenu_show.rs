use crate::{parse_maybe_u32, Parse, Values};
use rmpv::Value;
use serde::Serialize;

/// Show popupmenu completion
#[derive(Debug, Clone, Serialize)]
pub struct PopupmenuShow {
    /// The completion items to show
    pub items: Vec<Item>,
    /// The initially-selected item, if present
    pub selected: Option<u32>,
    /// The anchor position row
    pub row: u32,
    /// The anchor position col
    pub col: u32,
    /// The grid for the anchor position, unless the cmdline is externalized
    pub grid: Option<u32>,
}

impl Parse for PopupmenuShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            items: iter.next()?,
            selected: parse_maybe_u32(iter.next()?)?,
            row: iter.next()?,
            col: iter.next()?,
            grid: parse_maybe_u32(iter.next()?)?,
        })
    }
}

/// A popupmenu item
#[derive(Debug, Clone, Serialize)]
pub struct Item {
    /// The text that will be inserted
    pub word: String,
    /// Indicates the type of completion
    pub kind: Kind,
    /// Extra text for the popup menu, displayed after word
    pub menu: String,
    /// More information about the item. Can be displayed in a preview window.
    pub info: String,
}

impl Parse for Item {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            word: iter.next()?,
            kind: iter.next()?,
            menu: iter.next()?,
            info: iter.next()?,
        })
    }
}

/// Indicates the type of completion
#[derive(Debug, Clone, Serialize)]
pub enum Kind {
    /// Variable
    Variable,
    /// Function or method
    Function,
    /// Member of a struct or class
    Member,
    /// Typedef
    Typedef,
    /// #define or macro
    Define,
    /// Undocumented kind
    Other(String),
}

impl Parse for Kind {
    fn parse(value: Value) -> Option<Self> {
        let s = String::parse(value)?;
        Some(match s.as_str() {
            "v" => Self::Variable,
            "f" => Self::Function,
            "m" => Self::Member,
            "t" => Self::Typedef,
            "d" => Self::Define,
            _ => Self::Other(s),
        })
    }
}

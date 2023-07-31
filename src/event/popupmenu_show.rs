use super::util::{Parse, Values};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct PopupmenuShow {
    pub items: Vec<Item>,
    pub selected: i64,
    pub row: u64,
    pub col: u64,
    pub grid: i64,
}

impl Parse for PopupmenuShow {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            items: iter.next()?,
            selected: iter.next()?,
            row: iter.next()?,
            col: iter.next()?,
            grid: iter.next()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub word: String,
    pub kind: Kind,
    pub menu: String,
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

#[derive(Debug, Clone)]
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

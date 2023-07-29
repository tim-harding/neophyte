use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

#[derive(Debug, Clone)]
pub struct GridLine {
    pub grid: u64,
    pub row: u64,
    pub col_start: u64,
    pub cells: Vec<Cell>,
    // NOTE: There is supposedly a wrap argument that is supposed to go here but I don't know how
    // to make it show up.
}

impl GridLine {
    pub fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            grid: parse_u64(iter.next()?)?,
            row: parse_u64(iter.next()?)?,
            col_start: parse_u64(iter.next()?)?,
            cells: parse_array(iter.next()?)?
                .into_iter()
                .map(Cell::parse)
                .collect::<Option<Vec<_>>>()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub text: String,
    pub hl_id: Option<u64>,
    pub repeat: Option<u64>,
}

impl Cell {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = parse_array(value)?.into_iter();
        Some(Self {
            text: parse_string(iter.next()?)?,
            hl_id: iter.next().and_then(parse_u64),
            repeat: iter.next().and_then(parse_u64),
        })
    }
}

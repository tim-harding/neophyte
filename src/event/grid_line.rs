use super::util::{parse_array, parse_string, parse_u64};
use nvim_rs::Value;

/// Redraw a continuous part of a row on a grid.
#[derive(Debug, Clone)]
pub struct GridLine {
    /// The grid to draw on
    pub grid: u64,
    /// The row to draw
    pub row: u64,
    /// The column to start drawing on
    pub col_start: u64,
    /// The cells to draw
    pub cells: Vec<Cell>,
    // NOTE: There is supposedly a wrap argument that is supposed to go here but
    // I don't know how to make it show up.
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

/// A portion of a grid line to draw
#[derive(Debug, Clone)]
pub struct Cell {
    /// The text to draw.
    pub text: String,
    /// The highlight to apply to the text from a previous hl_attr_define event.
    /// If not present, use the most recent hl_id from the grid_line event.
    pub hl_id: Option<u64>,
    /// How many times to repeat the text, including the first time.
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

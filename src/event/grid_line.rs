use crate::util::{Parse, Values};
use rmpv::Value;

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

impl Parse for GridLine {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            grid: iter.next()?,
            row: iter.next()?,
            col_start: iter.next()?,
            cells: iter.next()?,
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

impl Parse for Cell {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        Some(Self {
            text: iter.next()?,
            hl_id: iter.next(),
            repeat: iter.next(),
        })
    }
}

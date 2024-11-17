use std::fmt::Debug;

use crate::{Parse, Values};
use rmpv::Value;

/// Redraw a continuous part of a row on a grid.
#[derive(Clone)]
pub struct GridLine {
    /// The grid to draw on
    pub grid: u32,
    /// The row to draw
    pub row: u16,
    /// The column to start drawing on
    pub col_start: u16,
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

impl Debug for GridLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("GridLine");
        s.field("grid", &self.grid);
        s.field("row", &self.row);
        s.field("col_start", &self.col_start);
        let cells: String = self
            .cells
            .iter()
            .flat_map(|cell| {
                std::iter::repeat(cell.text.chars())
                    .take(cell.repeat.unwrap_or(1) as usize)
                    .flatten()
            })
            .collect();
        s.field("cells", &cells);
        s.finish()
    }
}

/// A portion of a grid line to draw
#[derive(Debug, Clone)]
pub struct Cell {
    /// The text to draw.
    pub text: String,
    /// The highlight to apply to the text from a previous hl_attr_define event.
    /// If not present, use the most recent hl_id from the grid_line event.
    pub hl_id: Option<u32>,
    /// How many times to repeat the text, including the first time.
    pub repeat: Option<u32>,
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

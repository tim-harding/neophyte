#![allow(unused)]

use crate::util::Vec2;
use std::fmt::{self, Debug, Display, Formatter};

// TODO: Add fallback to string if the cell requires more than a char

#[derive(Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
    highlights: Vec<u64>,
    width: u64,
    height: u64,
}

impl Grid {
    pub fn resize(&mut self, size: Vec2) {
        // TODO: Resize in place
        let mut resized_cells = vec![' '; (size.x * size.y) as usize];
        let mut resized_hightlights = vec![0; (size.x * size.y) as usize];
        for y in 0..size.y.min(self.height) {
            for x in 0..size.x.min(self.width) {
                resized_cells[(y * size.x + x) as usize] =
                    self.cells[(y * self.width + x) as usize];
                resized_hightlights[(y * size.x + x) as usize] =
                    self.highlights[(y * self.width + x) as usize];
            }
        }
        self.width = size.x;
        self.height = size.y;
        self.cells = resized_cells;
        self.highlights = resized_hightlights;
    }

    pub fn get(&self, pos: Vec2) -> (char, u64) {
        let i = (pos.y * self.width + pos.x) as usize;
        (self.cells[i], self.highlights[i])
    }

    pub fn set(&mut self, pos: Vec2, c: char, highlight: u64) {
        let i = (pos.y * self.width + pos.x) as usize;
        self.cells[i] = c;
        self.highlights[i] = highlight;
    }

    pub fn clear(&mut self) {
        for (cell, highlight) in self.cells.iter_mut().zip(self.highlights.iter_mut()) {
            *cell = ' ';
            *highlight = 0;
        }
    }

    pub fn row(&self, i: u64) -> impl Iterator<Item = (char, u64)> + '_ {
        let w = self.width as usize;
        let start = i as usize * w;
        let end = start + w;
        self.cells[start..end]
            .iter()
            .cloned()
            .zip(self.highlights[start..end].iter().cloned())
    }

    pub fn row_mut(&mut self, i: u64) -> impl Iterator<Item = (&mut char, &mut u64)> + '_ {
        let w = self.width as usize;
        let start = i as usize * w;
        let end = start + w;
        self.cells[start..end]
            .iter_mut()
            .zip(self.highlights[start..end].iter_mut())
    }

    pub fn width(&self) -> u64 {
        self.width
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = (char, u64)> + '_> + '_ {
        self.cells
            .chunks(self.width as usize)
            .zip(self.highlights.chunks(self.width as usize))
            .map(|(cells_row, highlights_row)| {
                cells_row
                    .iter()
                    .cloned()
                    .zip(highlights_row.iter().cloned())
            })
    }

    pub fn rows_mut(
        &mut self,
    ) -> impl Iterator<Item = impl Iterator<Item = (&mut char, &mut u64)> + '_> + '_ {
        self.cells
            .chunks_mut(self.width as usize)
            .zip(self.highlights.chunks_mut(self.width as usize))
            .map(|(cells_row, highlights_row)| cells_row.iter_mut().zip(highlights_row.iter_mut()))
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "┏{:━<1$}┓\n", "", self.width as usize);
        for y in 0..self.height {
            write!(f, "┃");
            let row = self.row(y);
            for cell in row {
                let cell = cell.0;
                write!(f, "{cell}")?;
            }
            write!(f, "┃\n")?;
        }
        write!(f, "┗{:━<1$}┛", "", self.width as usize);
        Ok(())
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let row = self.row(y);
            for cell in row {
                let cell = cell.0;
                write!(f, "{cell}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

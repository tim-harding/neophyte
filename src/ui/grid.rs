#![allow(unused)]

use std::fmt::{self, Display, Formatter};

// TODO: Add fallback to string if the cell requires more than a char

#[derive(Debug, Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn resize(&mut self, width: usize, height: usize) {
        // TODO: Resize in place
        let mut new = vec![' '; (width * height) as usize];
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                new[(y * width + x) as usize] = self.cells[(y * self.width + x) as usize];
            }
        }
        self.width = width;
        self.height = height;
        self.cells = new;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = ' ';
        }
    }

    pub fn row(&self, i: usize) -> &[char] {
        let start = i * self.width;
        &self.cells[start..start + self.width]
    }

    pub fn row_mut(&mut self, i: usize) -> &mut [char] {
        let start = i * self.width;
        &mut self.cells[start..start + self.width]
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn rows(&self) -> impl Iterator<Item = &[char]> {
        self.cells.chunks(self.width)
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [char]> {
        self.cells.chunks_mut(self.width)
    }

    pub fn cells(&self) -> &[char] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [char] {
        &mut self.cells
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let row = self.row(y);
            for cell in row {
                write!(f, "{cell}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

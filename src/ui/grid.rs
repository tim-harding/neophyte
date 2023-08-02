#![allow(unused)]

use std::fmt::{self, Debug, Display, Formatter};

// TODO: Add fallback to string if the cell requires more than a char

#[derive(Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
    width: u64,
    height: u64,
}

impl Grid {
    pub fn resize(&mut self, width: u64, height: u64) {
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

    pub fn get(&self, x: u64, y: u64) -> char {
        let i = y * self.width + x;
        self.cells[i as usize]
    }

    pub fn set(&mut self, x: u64, y: u64, c: char) {
        let i = y * self.width + x;
        self.cells[i as usize] = c;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = ' ';
        }
    }

    pub fn row(&self, i: u64) -> &[char] {
        let w = self.width as usize;
        let start = i as usize * w;
        &self.cells[start..start + w]
    }

    pub fn row_mut(&mut self, i: u64) -> &mut [char] {
        let w = self.width as usize;
        let start = i as usize * w;
        &mut self.cells[start..start + w]
    }

    pub fn width(&self) -> u64 {
        self.width
    }

    pub fn height(&self) -> u64 {
        self.height
    }

    pub fn rows(&self) -> impl Iterator<Item = &[char]> {
        self.cells.chunks(self.width as usize)
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = &mut [char]> {
        self.cells.chunks_mut(self.width as usize)
    }

    pub fn cells(&self) -> &[char] {
        &self.cells
    }

    pub fn cells_mut(&mut self) -> &mut [char] {
        &mut self.cells
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "┏{:━<1$}┓\n", "", self.width as usize);
        for y in 0..self.height {
            write!(f, "┃");
            let row = self.row(y);
            for cell in row {
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
                write!(f, "{cell}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

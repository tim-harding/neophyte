#![allow(unused)]

use crate::event::hl_attr_define::Attributes;
use crate::event::{GridScroll, HlAttrDefine};
use crate::util::Vec2;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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

    pub fn combine(&mut self, other: Grid, anchor: Vec2) {
        for (y, row) in other.rows().enumerate() {
            for (x, (c, hl)) in row.into_iter().enumerate() {
                let pos = Vec2::new(x as u64, y as u64);
                self.set(pos + anchor, c, hl);
            }
        }
    }

    pub fn print_colored(&self, highlights: &HashMap<u64, HlAttrDefine>) {
        let mut f = StandardStream::stdout(ColorChoice::Always);
        let mut prev_hl = 0;
        writeln!(f, "┏{:━<1$}┓", "", self.width as usize);
        for y in 0..self.height {
            f.reset();
            write!(f, "┃");
            let row = self.row(y);
            for cell in row {
                let (c, hl) = cell;
                if hl != prev_hl {
                    if let Some(hl_attr) = highlights.get(&hl) {
                        f.set_color(&hl_attr_to_colorspec(hl_attr));
                    } else {
                        f.reset();
                    }
                    prev_hl = hl;
                }
                write!(f, "{c}");
            }
            f.reset();
            write!(f, "┃\n");
        }
        f.reset();
        writeln!(f, "┗{:━<1$}┛", "", self.width as usize);
    }

    pub fn scroll(&mut self, top: u64, bot: u64, left: u64, right: u64, rows: i64) {
        let height = self.height;
        let mut copy = move |src_y, dst_y| {
            for x in left..right {
                let (c, highlight) = self.get(Vec2::new(x, src_y));
                self.set(Vec2::new(x, dst_y), c, highlight);
            }
        };
        // TODO: Skip iterations for lines that won't be copied
        if rows > 0 {
            for y in top..bot {
                if let Ok(dst_y) = ((y as i64) - rows).try_into() {
                    copy(y, dst_y);
                }
            }
        } else {
            for y in (top..bot).rev() {
                let dst_y = ((y as i64) - rows) as u64;
                if dst_y < height {
                    copy(y, dst_y);
                }
            }
        }
    }
}

fn hl_attr_to_colorspec(hl: &HlAttrDefine) -> ColorSpec {
    let mut spec = ColorSpec::new();
    let hl = &hl.rgb_attr;
    let reverse = hl.reverse.unwrap_or(false);

    if let Some(foreground) = hl.foreground {
        let color = Some(u64_to_color(foreground));
        if reverse {
            spec.set_bg(color);
        } else {
            spec.set_fg(color);
        }
    }

    if let Some(background) = hl.background {
        let color = Some(u64_to_color(background));
        if reverse {
            spec.set_fg(color);
        } else {
            spec.set_bg(color);
        }
    }

    spec.set_italic(hl.italic.unwrap_or(false));
    spec.set_bold(hl.bold.unwrap_or(false));
    spec.set_strikethrough(hl.strikethrough.unwrap_or(false));
    spec.set_underline(hl.underline.unwrap_or(false));
    spec
}

fn u64_to_color(n: u64) -> Color {
    let r = (n >> 16) as u8;
    let g = (n >> 8) as u8;
    let b = n as u8;
    Color::Rgb(r, g, b)
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

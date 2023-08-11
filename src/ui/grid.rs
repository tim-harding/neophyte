#![allow(unused)]

use super::Highlights;
use crate::{
    event::{grid_line::Cell, hl_attr_define::Attributes, Anchor, GridScroll, HlAttrDefine},
    ui::print::hl_attr_to_colorspec,
    util::vec2::Vec2,
};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::Write,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

// TODO: Add fallback to string if the cell requires more than a char

pub type HighlightId = u32;

#[derive(Default, Clone)]
pub struct Grid {
    pub cells: InnerGrid<char>,
    pub highlights: InnerGrid<HighlightId>,
    pub show: bool,
    pub window: Window,
}

#[derive(Debug, Clone, Default)]
pub enum Window {
    #[default]
    None,
    Normal(NormalWindow),
    Floating(FloatingWindow),
    External,
}

#[derive(Debug, Clone)]
pub struct FloatingWindow {
    pub anchor: Anchor,
    pub anchor_grid: u64,
    pub anchor_pos: Vec2<f64>,
    pub focusable: bool,
}

#[derive(Debug, Clone)]
pub struct NormalWindow {
    pub start: Vec2<u64>,
    pub size: Vec2<u64>,
}

impl Grid {
    pub fn resize(&mut self, size: Vec2<u64>) {
        self.cells.resize(size);
        self.highlights.resize(size);
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.highlights.clear();
    }

    pub fn scroll(&mut self, top: u64, bot: u64, left: u64, right: u64, rows: i64) {
        self.cells.scroll(top, bot, left, right, rows);
        self.highlights.scroll(top, bot, left, right, rows);
    }

    pub fn size(&self) -> Vec2<u64> {
        self.cells.size
    }

    pub fn combine(&mut self, other: &Grid, cursor: Option<CursorRenderInfo>) {
        let start = match &other.window {
            Window::None => return,
            Window::External => return,
            Window::Normal(window) => window.start,
            Window::Floating(window) => {
                let anchor_pos = {
                    let (x, y) = window.anchor_pos.into();
                    Vec2::new(x.floor() as u64, y.floor() as u64)
                };
                // TODO: Should be relative to anchor grid
                anchor_pos
                    - other.size()
                        * match window.anchor {
                            Anchor::Nw => Vec2::new(0, 0),
                            Anchor::Ne => Vec2::new(0, 1),
                            Anchor::Sw => Vec2::new(1, 0),
                            Anchor::Se => Vec2::new(1, 1),
                        }
            }
        };

        self.cells.paste(&other.cells, start);
        self.highlights.paste(&other.highlights, start);

        // TODO: Take mode_info_set into consideration
        if let Some(cursor) = cursor {
            let pos = start + cursor.pos;
            let i = self.highlights.index_for(pos);
            self.highlights.buffer[i] = cursor.hl;
        }
    }

    pub fn print_colored(&self, highlights: &Highlights) {
        let mut f = StandardStream::stdout(ColorChoice::Always);
        let mut prev_hl = 0;
        writeln!(f, "┏{:━<1$}┓", "", self.size().x as usize);
        for (cell_row, hl_row) in self.cells.rows().zip(self.highlights.rows()) {
            f.reset();
            write!(f, "┃");
            for (c, hl) in cell_row.into_iter().zip(hl_row.into_iter()) {
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
        writeln!(f, "┗{:━<1$}┛", "", self.size().x as usize);
    }

    pub fn grid_line(&mut self, row: u64, col_start: u64, cells: Vec<Cell>) {
        // TODO: Apply changes to glyph quads
        let mut row = self
            .cells
            .row_mut(row)
            .into_iter()
            .zip(self.highlights.row_mut(row).into_iter())
            .skip(col_start as usize);
        let mut highlight = 0;
        for cell in cells {
            let c = cell.text.chars().into_iter().next().unwrap();
            if let Some(hl_id) = cell.hl_id {
                highlight = hl_id as HighlightId + 1;
            }
            // TODO: Skip iterations for lines that won't be copied
            if let Some(repeat) = cell.repeat {
                for _ in 0..repeat {
                    let dst = row.next().unwrap();
                    *dst.0 = c;
                    *dst.1 = highlight;
                }
            } else {
                let dst = row.next().unwrap();
                *dst.0 = c;
                *dst.1 = highlight;
            }
        }
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "┏{:━<1$}┓\n", "", self.size().x as usize);
        for row in self.cells.rows() {
            write!(f, "┃");
            for cell in row {
                write!(f, "{cell}")?;
            }
            write!(f, "┃\n")?;
        }
        write!(f, "┗{:━<1$}┛", "", self.size().x as usize);
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InnerGrid<T>
where
    T: Debug + Copy + Clone + Send + Sync + Default + 'static,
{
    size: Vec2<u64>,
    buffer: Vec<T>,
}

impl<T> InnerGrid<T>
where
    T: Debug + Copy + Clone + Send + Sync + Default + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resize(&mut self, size: Vec2<u64>) {
        let old = std::mem::take(&mut self.buffer);
        self.buffer = vec![T::default(); size.area() as usize];
        if self.size.x > 0 {
            for (src, dst) in old
                .chunks(self.size.x as usize)
                .zip(self.buffer.chunks_mut(size.x as usize))
            {
                for (src, dst) in src.into_iter().zip(dst.iter_mut()) {
                    *dst = *src;
                }
            }
        }
        self.size = size;
    }

    pub fn index_for(&self, position: Vec2<u64>) -> usize {
        (position.y * self.size.x + position.x) as usize
    }

    pub fn get(&self, position: Vec2<u64>) -> T {
        self.buffer[self.index_for(position)]
    }

    pub fn set(&mut self, position: Vec2<u64>, value: T) {
        let i = self.index_for(position);
        self.buffer[i] = value;
    }

    pub fn clear(&mut self) {
        for dst in self.buffer.iter_mut() {
            *dst = T::default();
        }
    }

    pub fn scroll(&mut self, top: u64, bot: u64, left: u64, right: u64, rows: i64) {
        // TODO: Skip iterations for lines that won't be copied
        // TODO: Maybe use chunks and iterators?
        let height = self.size.y;
        let mut copy = move |src_y, dst_y| {
            for x in left..right {
                let t = self.get(Vec2::new(x, src_y));
                self.set(Vec2::new(x, dst_y), t);
            }
        };
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

    pub fn row(&self, i: u64) -> impl Iterator<Item = T> + '_ {
        let w = self.size.x as usize;
        let start = i as usize * w;
        let end = start + w;
        self.buffer[start..end].iter().cloned()
    }

    pub fn row_mut(&mut self, i: u64) -> impl Iterator<Item = &mut T> + '_ {
        let w = self.size.x as usize;
        let start = i as usize * w;
        let end = start + w;
        self.buffer[start..end].iter_mut()
    }

    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = T> + '_> + '_ {
        self.buffer
            .chunks(self.size.x as usize)
            .map(|chunk| chunk.into_iter().cloned())
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = impl Iterator<Item = &mut T> + '_> + '_ {
        self.buffer
            .chunks_mut(self.size.x as usize)
            .map(|chunk| chunk.iter_mut())
    }

    pub fn cells(&self) -> impl Iterator<Item = T> + '_ {
        self.buffer.iter().cloned()
    }

    pub fn paste(&mut self, other: &Self, offset: Vec2<u64>) {
        for (src, dst) in other.rows().zip(self.rows_mut().skip(offset.y as usize)) {
            for (src, dst) in src.into_iter().zip(dst.into_iter().skip(offset.x as usize)) {
                *dst = src;
            }
        }
    }
}

pub struct CursorRenderInfo {
    pub hl: HighlightId,
    pub pos: Vec2<u64>,
}

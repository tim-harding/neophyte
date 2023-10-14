#![allow(unused)]

use super::{
    packed_char::{PackedChar, U22},
    window::{Window, WindowOffset},
};
use crate::{
    event::{grid_line, hl_attr_define::Attributes, Anchor, GridScroll, HlAttrDefine},
    ui::packed_char::PackedCharContents,
    util::vec2::Vec2,
};
use bitfield_struct::bitfield;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::Write,
    marker::PhantomData,
    ops::Not,
    str::Chars,
    vec::IntoIter,
};

#[derive(Debug, Default, Clone)]
pub struct Grid {
    pub id: u64,
    pub scroll_delta: i64,
    pub dirty: DirtyFlags,
    window: Window,
    contents: GridContents,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct DirtyFlags {
    pub contents: bool,
    pub window: bool,
    #[bits(6)]
    __: u8,
}

impl Grid {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn contents(&self) -> &GridContents {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut GridContents {
        self.dirty.set_contents(true);
        &mut self.contents
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        self.dirty.set_window(true);
        &mut self.window
    }

    pub fn clear_dirty(&mut self) {
        self.dirty.set_window(false);
        self.dirty.set_contents(false);
        self.scroll_delta = 0;
    }
}

#[derive(Default, Clone)]
pub struct GridContents {
    pub size: Vec2<u64>,
    pub buffer: Vec<Cell>,
    pub overflow: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cell {
    pub text: PackedChar,
    pub highlight: u32,
}

impl GridContents {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resize(&mut self, size: Vec2<u64>) {
        let mut old = std::mem::take(&mut self.buffer).into_iter();
        self.buffer = vec![Cell::default(); size.area() as usize];
        for y in 0..self.size.y.min(size.y) {
            let offset = y * size.x;
            for x in 0..self.size.x.min(size.x) {
                self.buffer[(offset + x) as usize] = old.next().unwrap();
            }
            for _ in size.x..self.size.x {
                let _ = old.next();
            }
        }
        self.size = size;
    }

    pub fn scroll(&mut self, top: u64, bot: u64, left: u64, right: u64, rows: i64) {
        let height = self.size.y;
        let mut cut_and_paste = move |src_y, dst_y| {
            for x in left..right {
                let t = self.take(Vec2::new(x, src_y));
                self.set(Vec2::new(x, dst_y), t);
            }
        };
        let dst_top = top as i64 - rows;
        let dst_bot = bot as i64 - rows;
        if rows > 0 {
            for dst_y in dst_top.max(0)..dst_bot.max(0) {
                let y = dst_y + rows;
                cut_and_paste(y as u64, dst_y as u64);
            }
        } else {
            let dst_top = dst_top.min(height as i64);
            let dst_bot = dst_bot.min(height as i64);
            for dst_y in (dst_top..dst_bot).rev() {
                cut_and_paste((dst_y + rows) as u64, dst_y as u64);
            }
        }
    }

    pub fn grid_line(&mut self, row: u64, col_start: u64, cells: Vec<grid_line::Cell>) {
        // Inlined self.row_mut() to satisfy borrow checker
        let w = self.size.x as usize;
        let start = row as usize * w;
        let end = start + w;
        let mut row = self.buffer[start..end].iter_mut().skip(col_start as usize);

        let mut highlight = 0;
        for cell in cells {
            if let Some(hl_id) = cell.hl_id {
                highlight = hl_id;
            }

            let repeat = cell.repeat.unwrap_or(1);
            let mut chars = cell.text.chars();
            let packed = match (chars.next(), chars.next()) {
                (None, None) => PackedChar::from_char(' '),
                (None, Some(c)) => unreachable!(),
                (Some(c), None) => PackedChar::from_char(c),
                (Some(c1), Some(c2)) => {
                    let i: u32 = self.overflow.len().try_into().unwrap();
                    self.overflow.push(cell.text);
                    PackedChar::from_u22(U22::from_u32(i).unwrap())
                }
            };
            let cell = Cell {
                text: packed,
                highlight: highlight.try_into().unwrap(),
            };

            for _ in 0..repeat {
                *row.next().unwrap() = cell;
            }
        }
    }

    pub fn index_for(&self, position: Vec2<u64>) -> usize {
        (position.y * self.size.x + position.x) as usize
    }

    pub fn get(&self, position: Vec2<u64>) -> &Cell {
        &self.buffer[self.index_for(position)]
    }

    pub fn take(&mut self, position: Vec2<u64>) -> Cell {
        let i = self.index_for(position);
        std::mem::take(&mut self.buffer[i])
    }

    pub fn set(&mut self, position: Vec2<u64>, value: Cell) {
        let i = self.index_for(position);
        self.buffer[i] = value;
    }

    pub fn clear(&mut self) {
        for dst in self.buffer.iter_mut() {
            *dst = Cell::default();
        }
    }

    pub fn rows<'a>(
        &'a self,
    ) -> impl Iterator<Item = impl Iterator<Item = CellContents<'a>> + '_ + Clone> + '_ + Clone
    {
        self.buffer.chunks(self.size.x as usize).map(|chunk| {
            chunk.iter().map(|cell| {
                let text = match cell.text.contents() {
                    PackedCharContents::Char(c) => c.into(),
                    PackedCharContents::U22(u22) => {
                        self.overflow[u22.as_u32() as usize].chars().into()
                    }
                };
                CellContents {
                    text,
                    highlight: cell.highlight,
                }
            })
        })
    }

    pub fn copy_from(&mut self, other: &GridContents) {
        self.size = other.size;
        self.overflow
            .extend_from_slice(&other.overflow[self.overflow.len()..]);
        self.buffer.resize(other.buffer.len(), Cell::default());
        self.buffer.copy_from_slice(other.buffer.as_slice());
    }
}

impl Debug for GridContents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "┏{:━<1$}┓", "", self.size.x as usize);
        for row in self.rows() {
            write!(f, "┃");
            for mut cell in row {
                let c = cell
                    .text
                    .next()
                    .map(|c| if c == '\0' { ' ' } else { c })
                    .unwrap_or(' ');
                write!(f, "{}", c)?;
            }
            writeln!(f, "┃")?;
        }
        write!(f, "┗{:━<1$}┛", "", self.size.x as usize);
        Ok(())
    }
}

#[derive(Clone)]
pub enum OnceOrChars<'a> {
    Char(std::iter::Once<char>),
    Chars(Chars<'a>),
}

impl<'a> From<char> for OnceOrChars<'a> {
    fn from(c: char) -> Self {
        Self::Char(std::iter::once(c))
    }
}

impl<'a> From<Chars<'a>> for OnceOrChars<'a> {
    fn from(chars: Chars<'a>) -> Self {
        Self::Chars(chars)
    }
}

impl<'a> Iterator for OnceOrChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OnceOrChars::Char(iter) => iter.next(),
            OnceOrChars::Chars(iter) => iter.next(),
        }
    }
}

pub struct CellContents<'a> {
    pub highlight: u32,
    pub text: OnceOrChars<'a>,
}

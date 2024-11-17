#![allow(unused)]

use super::{
    window::{Window, WindowOffset},
    HlId,
};
use neophyte_linalg::{CellVec, Vec2};
use neophyte_ui_event::{grid_line, hl_attr_define::Attributes, Anchor, GridScroll, HlAttrDefine};
use packed_char::{Contents, PackedChar, U22};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::Write,
    iter::TakeWhile,
    marker::PhantomData,
    mem::transmute,
    ops::Not,
    str::Chars,
    vec::IntoIter,
};

pub type Id = u32;

#[derive(Debug, Default, Clone)]
pub struct Grid {
    pub id: Id,
    pub scroll_delta: i32,
    pub dirty: DirtyFlags,
    window: Window,
    contents: GridContents,
}

#[derive(PartialEq, Eq, Debug, Default, Clone, Copy, PartialOrd, Ord)]
pub struct DirtyFlags(u8);

#[rustfmt::skip]
impl DirtyFlags {
    const CONTENTS: u8 = 0b01;
    const WINDOW:   u8 = 0b10;
}

impl DirtyFlags {
    pub fn set_contents(&mut self) {
        self.0 |= Self::CONTENTS
    }

    pub fn contents(self) -> bool {
        self.0 & Self::CONTENTS > 0
    }

    pub fn set_window(&mut self) {
        self.0 |= Self::WINDOW
    }

    pub fn window(self) -> bool {
        self.0 & Self::WINDOW > 0
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

impl Grid {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn contents(&self) -> &GridContents {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut GridContents {
        self.dirty.set_contents();
        &mut self.contents
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        self.dirty.set_window();
        &mut self.window
    }

    /// Reset dirty flags and scroll delta
    pub fn clear_dirty(&mut self) {
        self.dirty.clear();
        self.scroll_delta = 0;
    }
}

/// The contents of a grid
#[derive(Default, Clone)]
pub struct GridContents {
    /// Grid dimensions
    pub size: CellVec<u16>,
    /// Grid cells in rows then columns
    buffer: Vec<Cell>,
    /// Contains cell contents for cells that require more than one char of
    /// storage. This optimizes for the common case by keeping the main buffer
    /// tightly packed.
    overflow: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cell {
    pub text: PackedChar,
    pub highlight: HlId,
}

impl GridContents {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resize the grid to the new dimensions
    pub fn resize(&mut self, size: CellVec<u16>) {
        let mut old = std::mem::take(&mut self.buffer);
        self.buffer = vec![Cell::default(); size.0.area() as usize];
        for (new, old) in self
            .buffer
            .chunks_mut(size.0.x as usize)
            .zip(old.chunks(self.size.0.x.max(1) as usize))
        {
            for (new, old) in new.iter_mut().zip(old.iter()) {
                *new = *old;
            }
        }
        self.size = size;
    }

    /// Apply a grid_scroll event
    pub fn scroll(&mut self, top: u16, bot: u16, left: u16, right: u16, rows: i32) {
        let left = left as usize;
        let right = right as usize;
        let size: Vec2<usize> = self.size.0.cast_as();
        let dst_top = top as i32 - rows;
        let dst_bot = bot as i32 - rows;
        if rows > 0 {
            // Move a region up
            let rows = rows as usize;
            let top = top as usize + rows;
            let bot = bot as usize;
            for src_y in top..bot {
                let dst_y = src_y - rows;
                let (dst, src) = self.buffer.split_at_mut(src_y * size.x);
                let dst = &mut dst[dst_y * size.x..];
                let dst = &mut dst[left..right];
                let src = &src[left..right];
                dst.copy_from_slice(src);
            }
        } else {
            // Move a region down
            let rows = (-rows) as usize;
            let top = top as usize;
            let bot = bot as usize - rows;
            for src_y in (top..bot).rev() {
                let dst_y = src_y + rows;
                let (src, dst) = self.buffer.split_at_mut(dst_y * size.x);
                let src = &src[src_y * size.x..];
                let src = &src[left..right];
                let dst = &mut dst[left..right];
                dst.copy_from_slice(src);
            }
        }
    }

    /// Apply a grid_line event
    pub fn grid_line(&mut self, row: u16, col_start: u16, cells: Vec<grid_line::Cell>) {
        let w = self.size.0.x as usize;
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
                (None, None) => PackedChar::from_char('\0'),
                (None, Some(c)) => unreachable!(),
                (Some(c), None) => PackedChar::from_char(c),
                (Some(c1), Some(c2)) => {
                    let i = self.overflow.len().try_into().unwrap();
                    self.overflow.push(cell.text);
                    PackedChar::from_u22(U22::from_u32(i).unwrap())
                }
            };
            let cell = Cell {
                text: packed,
                highlight,
            };

            for _ in 0..repeat {
                *row.next().unwrap() = cell;
            }
        }
    }

    /// Reset the contents of the grid
    pub fn clear(&mut self) {
        for dst in self.buffer.iter_mut() {
            *dst = Cell::default();
        }
    }

    /// Iterate over the grid contents row by row
    pub fn rows(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = CellContents<'_>> + '_ + Clone> + '_ + Clone
    {
        self.buffer.chunks(self.size.0.x as usize).map(|chunk| {
            chunk.iter().map(|cell| {
                let text = match cell.text.contents() {
                    Contents::Char(c) => c.into(),
                    Contents::U22(u22) => self.overflow[u22.as_u32() as usize].chars().into(),
                };
                CellContents {
                    text,
                    highlight: cell.highlight,
                }
            })
        })
    }
}

impl Debug for GridContents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "┏{:━<1$}┓", "", self.size.0.x as usize);
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
        write!(f, "┗{:━<1$}┛", "", self.size.0.x as usize);
        Ok(())
    }
}

#[derive(Clone)]
pub enum OnceOrChars<'a> {
    Char(std::iter::Once<char>),
    Chars(std::str::Chars<'a>),
}

impl<'a> From<char> for OnceOrChars<'a> {
    fn from(c: char) -> Self {
        Self::Char(std::iter::once(c))
    }
}

impl<'a> From<std::str::Chars<'a>> for OnceOrChars<'a> {
    fn from(value: std::str::Chars<'a>) -> Self {
        Self::Chars(value)
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

#[derive(Clone)]
pub struct CellContents<'a> {
    pub highlight: u32,
    pub text: OnceOrChars<'a>,
}

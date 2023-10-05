#![allow(unused)]

use bitfield_struct::bitfield;
use compact_str::CompactString;

use super::{
    packed_char::{PackedChar, U22},
    window::{Window, WindowOffset},
    Highlights,
};
use crate::{
    event::{grid_line, hl_attr_define::Attributes, Anchor, GridScroll, HlAttrDefine},
    ui::packed_char::PackedCharContents,
    util::vec2::Vec2,
};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::Write,
    marker::PhantomData,
    ops::Not,
    vec::IntoIter,
};

#[derive(Default, Clone)]
pub struct Grid {
    pub size: Vec2<u64>,
    pub buffer: Vec<Cell>,
    pub overflow: Vec<CompactString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub text: PackedChar,
    pub highlight: u32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: PackedChar::from_char('\0'),
            highlight: 0,
        }
    }
}

impl Grid {
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
                    self.overflow.push(cell.text.into());
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

    pub fn row(&self, i: u64) -> impl Iterator<Item = &Cell> + '_ {
        let w = self.size.x as usize;
        let start = i as usize * w;
        let end = start + w;
        self.buffer[start..end].iter()
    }

    pub fn row_mut(&mut self, i: u64) -> impl Iterator<Item = &mut Cell> + '_ {
        let w = self.size.x as usize;
        let start = i as usize * w;
        let end = start + w;
        self.buffer[start..end].iter_mut()
    }

    pub fn rows(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = &Cell> + '_ + Clone> + '_ + Clone {
        self.buffer
            .chunks(self.size.x as usize)
            .map(|chunk| chunk.iter())
    }

    pub fn rows_mut(&mut self) -> impl Iterator<Item = impl Iterator<Item = &mut Cell> + '_> + '_ {
        self.buffer
            .chunks_mut(self.size.x as usize)
            .map(|chunk| chunk.iter_mut())
    }

    pub fn combine(&mut self, other: Grid, cursor: Option<CursorRenderInfo>, start: Vec2<i64>) {
        let mut iter = other.buffer.into_iter();
        let size_x = self.size.x;
        for dst in self
            .rows_mut()
            .skip(start.y as usize)
            .take(other.size.y as usize)
        {
            for dst in dst
                .into_iter()
                .skip(start.x as usize)
                .take(other.size.x as usize)
            {
                *dst = iter.next().unwrap();
            }
        }

        if let Some(cursor) = cursor {
            let cursor_pos = cursor.pos.try_cast::<i64>().unwrap();
            let pos = cursor_pos + start;
            if let Ok(pos) = pos.try_cast() {
                let i = self.index_for(pos);
                self.buffer[i].highlight = cursor.hl.try_into().unwrap();
            }
        }
    }

    pub fn copy_from(&mut self, other: &Grid) {
        self.size = other.size;
        self.overflow
            .extend_from_slice(&other.overflow[self.overflow.len()..]);
        self.buffer.resize(other.buffer.len(), Cell::default());
        self.buffer.copy_from_slice(other.buffer.as_slice());
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "┏{:━<1$}┓", "", self.size.x as usize);
        for row in self.rows() {
            write!(f, "┃");
            for cell in row {
                match cell.text.contents() {
                    PackedCharContents::Char(c) => {
                        write!(f, "{}", c)?;
                    }
                    PackedCharContents::U22(u22) => {
                        let s = &self.overflow[u22.as_u32() as usize];
                        write!(f, "{s}");
                    }
                }
            }
            writeln!(f, "┃")?;
        }
        write!(f, "┗{:━<1$}┛", "", self.size.x as usize);
        Ok(())
    }
}

pub struct CursorRenderInfo {
    pub hl: u64,
    pub pos: Vec2<u64>,
}

#[derive(Debug, Default)]
pub struct DoubleBufferGrid {
    pub id: u64,
    pub scroll_delta: i64,
    dirty: DirtyFlags,
    window: Window,
    current: Grid,
    previous: Grid,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct DirtyFlags {
    pub grid: bool,
    pub window: bool,
    #[bits(6)]
    __: u8,
}

impl DoubleBufferGrid {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn current(&self) -> &Grid {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut Grid {
        self.dirty.set_grid(true);
        &mut self.current
    }

    pub fn previous(&self) -> &Grid {
        &self.previous
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut Window {
        self.dirty.set_window(true);
        &mut self.window
    }

    pub fn is_grid_dirty(&self) -> bool {
        self.dirty.grid()
    }

    pub fn is_window_dirty(&self) -> bool {
        self.dirty.window()
    }

    pub fn flush(&mut self) {
        if self.dirty.grid() {
            self.previous.copy_from(&self.current);
        }
        self.dirty.set_window(false);
        self.dirty.set_grid(false);
        self.scroll_delta = 0;
    }
}

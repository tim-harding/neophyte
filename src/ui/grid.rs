#![allow(unused)]

use super::{
    window::{Window, WindowOffset},
    Highlights,
};
use crate::{
    event::{grid_line, hl_attr_define::Attributes, Anchor, GridScroll, HlAttrDefine},
    util::vec2::Vec2,
};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display, Formatter},
    io::Write,
    marker::PhantomData,
    vec::IntoIter,
};

#[derive(Default, Clone)]
pub struct Grid {
    pub id: u64,
    pub size: Vec2<u64>,
    pub buffer: Vec<Cell>,
    pub window: Window,
    pub dirty: bool,
    pub scroll_delta: i64,
}

#[derive(Default, Clone)]
pub struct Cell {
    pub text: String,
    pub highlight: u64,
}

impl Grid {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn resize(&mut self, size: Vec2<u64>) {
        self.set_dirty();
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
        self.set_dirty();
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
        self.set_dirty();
        let mut row = self.row_mut(row).skip(col_start as usize);
        let mut highlight = 0;
        for cell in cells {
            if let Some(hl_id) = cell.hl_id {
                highlight = hl_id;
            }
            if let Some(repeat) = cell.repeat {
                if repeat == 0 {
                    continue;
                }
                for _ in 0..repeat - 1 {
                    let dst = row.next().unwrap();
                    dst.text = cell.text.clone();
                    dst.highlight = highlight;
                }
            }
            let dst = row.next().unwrap();
            dst.text = cell.text;
            dst.highlight = highlight;
        }
    }

    fn set_dirty(&mut self) {
        self.dirty = true;
        self.scroll_delta = 0;
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

    pub fn offset(&self) -> WindowOffset {
        self.window.offset(self.size)
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
                self.buffer[i].highlight = cursor.hl;
            }
        }
    }
}

impl Debug for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "┏{:━<1$}┓", "", self.size.x as usize);
        for row in self.rows() {
            write!(f, "┃");
            for cell in row {
                let c = if cell.text.is_empty() {
                    " "
                } else {
                    cell.text.as_str()
                };
                write!(f, "{}", cell.text)?;
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

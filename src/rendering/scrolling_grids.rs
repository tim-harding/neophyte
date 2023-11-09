use super::range::Range;
use crate::{
    rendering::Motion,
    ui::grid::{CellContents, GridContents},
    util::vec2::Vec2,
};

// TODO: Should move scrolling grids and range into a module together

pub struct ScrollingGrids {
    scrolling: Vec<GridPart>,
    t: f32,
}

impl ScrollingGrids {
    #[allow(unused)]
    pub fn new(grid: GridContents) -> Self {
        Self {
            scrolling: vec![GridPart::new(grid)],
            t: 0.,
        }
    }

    pub fn finish_scroll(&mut self) {
        self.scrolling.drain(0..self.scrolling.len() - 1);
        assert_eq!(self.scrolling.len(), 1);
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn advance(&mut self, delta_seconds: f32) -> Motion {
        if self.t.abs() < 0.025 {
            self.t = 0.0;
            self.finish_scroll();
            Motion::Still
        } else {
            let sign = if self.t.is_sign_positive() { -1.0 } else { 1.0 };
            let mag = self.t.abs() * delta_seconds + 0.25;
            let mag = mag.ln_1p().powf(1.5);
            let mag = mag.min(self.t.abs());
            self.t += sign * mag;
            Motion::Animating
        }
    }

    pub fn push(&mut self, grid: GridContents, offset: i64) {
        let sign = if offset.is_positive() { 1 } else { -1 };
        let mag = offset.abs().min(grid.size.y.try_into().unwrap());
        let offset = mag * sign;
        let mut cover = Range::until(grid.size.y as i64);
        self.t += offset as f32;
        self.scrolling.retain_mut(|part| {
            part.offset -= offset;
            let grid_range = Range::until(part.grid.size.y as i64) + part.offset;
            let covered = grid_range.cover(cover);
            cover = cover.union(grid_range);
            let grid_range = covered - part.offset;
            part.start = grid_range.start.try_into().unwrap();
            part.end = grid_range.end.try_into().unwrap();
            !part.is_empty()
        });
        self.scrolling.push(GridPart::new(grid));
    }

    pub fn replace_last(&mut self, grid: GridContents) {
        *self.scrolling.last_mut().unwrap() = GridPart::new(grid);
    }

    pub fn rows<'a, 'b: 'a>(
        &'a self,
    ) -> impl Iterator<Item = (i64, impl Iterator<Item = CellContents<'a>> + '_ + Clone)> + '_ + Clone
    {
        self.scrolling.iter().rev().flat_map(|part| {
            part.grid
                .rows()
                .enumerate()
                .skip(part.start)
                .take(part.end - part.start)
                .map(|(i, cells)| (i as i64 + part.offset, cells))
        })
    }

    pub fn size(&self) -> Vec2<u64> {
        self.scrolling.last().unwrap().grid.size
    }

    pub fn offset(&self, cell_height: f32) -> Vec2<i32> {
        Vec2::new(0, (self.t() * cell_height) as i32)
    }
}

struct GridPart {
    grid: GridContents,
    offset: i64,
    start: usize,
    end: usize,
}

impl GridPart {
    pub fn new(grid: GridContents) -> Self {
        Self {
            offset: 0,
            start: 0,
            end: grid.size.y as usize,
            grid,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

mod range;

use crate::{
    rendering::Motion,
    ui::grid::{CellContents, GridContents},
    util::nice_s_curve,
};
use neophyte_linalg::CellVec;
use range::Range;
use std::{collections::VecDeque, time::Duration};

pub struct ScrollingGrids {
    scrolling: VecDeque<GridPart>,
    t: Duration,
    offset_start: f32,
}

impl ScrollingGrids {
    #[allow(unused)]
    pub fn new(grid: GridContents) -> Self {
        let mut scrolling = VecDeque::new();
        scrolling.push_back(GridPart::new(grid));
        Self {
            scrolling,
            t: Duration::ZERO,
            offset_start: 0.,
        }
    }

    pub fn finish_scroll(&mut self) {
        self.scrolling.drain(1..);
        assert_eq!(self.scrolling.len(), 1);
    }

    pub fn advance(&mut self, delta_time: Duration, speed: f32) -> Motion {
        self.t +=
            Duration::try_from_secs_f32(delta_time.as_secs_f32() * speed).unwrap_or(Duration::ZERO);
        if self.offset_y() == 0. {
            self.finish_scroll();
            Motion::Still
        } else {
            Motion::Animating
        }
    }

    pub fn push(&mut self, grid: GridContents, offset: i32) {
        // TODO: Add desired screen region
        let sign = if offset.is_positive() { 1 } else { -1 };
        let mag = offset.abs().min(grid.size.0.y.into());
        let offset = mag * sign;
        let mut coverage = Range::until(grid.size.0.y.into());
        self.offset_start = self.offset_y() + offset as f32;
        self.t = Duration::ZERO;
        self.scrolling.retain_mut(|part| {
            part.offset -= offset;
            let grid_range = Range::until(part.grid.size.0.y.into()) + part.offset;
            let uncovered = grid_range.cover(coverage);
            coverage = coverage.union(grid_range);
            if let Some(uncovered) = uncovered {
                let grid_range = uncovered - part.offset;
                part.start = grid_range.start.try_into().unwrap();
                part.end = grid_range.end.try_into().unwrap();
                // Useful when resizing the window
                part.grid.size.0.y == grid.size.0.y
            } else {
                false
            }
        });
        self.scrolling.push_front(GridPart::new(grid));
    }

    pub fn replace(&mut self, grid: GridContents) {
        *self.scrolling.front_mut().unwrap() = GridPart::new(grid);
    }

    pub fn rows<'a, 'b: 'a>(
        &'a self,
    ) -> impl Iterator<Item = (i32, impl Iterator<Item = CellContents<'a>> + 'a + Clone)> + 'a + Clone
    {
        self.scrolling.iter().rev().flat_map(|part| {
            part.grid
                .rows()
                .enumerate()
                .skip(part.start)
                .take(part.end - part.start)
                .map(|(i, cells)| (i as i32 + part.offset, cells))
        })
    }

    pub fn size(&self) -> CellVec<u16> {
        self.scrolling.back().unwrap().grid.size
    }

    fn offset_y(&self) -> f32 {
        self.offset_start * (1. - nice_s_curve(self.t.as_secs_f32(), self.offset_start.abs()))
    }

    pub fn offset(&self) -> CellVec<f32> {
        CellVec::new(0., self.offset_y())
    }
}

struct GridPart {
    grid: GridContents,
    offset: i32,
    start: usize,
    end: usize,
}

impl GridPart {
    pub fn new(grid: GridContents) -> Self {
        Self {
            offset: 0,
            start: 0,
            end: grid.size.0.y as usize,
            grid,
        }
    }
}

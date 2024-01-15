use super::grid;
use crate::{
    event::Anchor,
    util::vec2::{CellVec, Vec2},
};

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub enum Window {
    #[default]
    None,
    Normal(NormalWindow),
    Floating(FloatingWindow),
    Messages {
        row: u16,
    },
    External,
}

impl Window {
    pub fn offset(&self, grid_size: CellVec<u16>) -> WindowOffset {
        match &self {
            Window::None => Default::default(),
            Window::External => Default::default(),
            Window::Messages { row } => WindowOffset {
                offset: CellVec(Vec2::new(0, *row).cast_as()),
                anchor_grid: None,
            },
            Window::Normal(window) => WindowOffset {
                offset: window.start.cast_as(),
                anchor_grid: None,
            },
            Window::Floating(window) => {
                let offset = grid_size
                    * match window.anchor {
                        Anchor::Nw => CellVec(Vec2::new(0, 0)),
                        Anchor::Ne => CellVec(Vec2::new(1, 0)),
                        Anchor::Sw => CellVec(Vec2::new(0, 1)),
                        Anchor::Se => CellVec(Vec2::new(1, 1)),
                    };
                WindowOffset {
                    offset: window.anchor_pos - offset.cast_as(),
                    anchor_grid: Some(window.anchor_grid),
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindowOffset {
    pub offset: CellVec<f32>,
    pub anchor_grid: Option<grid::Id>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FloatingWindow {
    pub anchor: Anchor,
    pub anchor_grid: grid::Id,
    pub anchor_pos: CellVec<f32>,
    pub focusable: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NormalWindow {
    pub start: CellVec<u16>,
    pub size: CellVec<u16>,
}

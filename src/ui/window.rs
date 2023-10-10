use crate::{event::Anchor, util::vec2::Vec2};

#[derive(Debug, Copy, Clone, Default)]
pub enum Window {
    #[default]
    None,
    Normal(NormalWindow),
    Floating(FloatingWindow),
    External,
}

impl Window {
    pub fn offset(&self, grid_size: Vec2<u64>) -> WindowOffset {
        match &self {
            Window::None => Default::default(),
            Window::External => Default::default(),
            Window::Normal(window) => WindowOffset {
                offset: window.start.cast_as(),
                anchor_grid: None,
            },
            Window::Floating(window) => {
                let offset = grid_size
                    * match window.anchor {
                        Anchor::Nw => Vec2::new(0, 0),
                        Anchor::Ne => Vec2::new(1, 0),
                        Anchor::Sw => Vec2::new(0, 1),
                        Anchor::Se => Vec2::new(1, 1),
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
    pub offset: Vec2<f64>,
    pub anchor_grid: Option<u64>,
}

#[derive(Debug, Copy, Clone)]
pub struct FloatingWindow {
    pub anchor: Anchor,
    pub anchor_grid: u64,
    pub anchor_pos: Vec2<f64>,
    pub focusable: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct NormalWindow {
    pub start: Vec2<u64>,
    pub size: Vec2<u64>,
}

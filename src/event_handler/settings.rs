use crate::util::vec2::Vec2;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// Multiplier of the default cursor speed
    pub cursor_speed: f32,
    /// Multiplier of the default scroll speed
    pub scroll_speed: f32,
    /// Additional offset to apply to underlines
    pub underline_offset: i32,
    /// The size of the render surface, or None to use the default
    pub render_size: Option<Vec2<u32>>,
    /// The directory to save frames to, or None if not rendering
    pub render_target: Option<PathBuf>,
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cursor_speed: 1.,
            scroll_speed: 1.,
            underline_offset: 2,
            render_size: None,
            render_target: None,
        }
    }
}

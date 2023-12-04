use crate::util::vec2::PixelVec;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// Multiplier of the default cursor speed
    pub cursor_speed: f32,
    /// Multiplier of the default scroll speed
    pub scroll_speed: f32,
    /// The size of the render surface, or None to use the default
    pub render_size: Option<PixelVec<u32>>,
    /// The directory to save frames to, or None if not rendering
    pub render_target: Option<PathBuf>,
    /// Overrides the background from Neovim's DefaultColorsSet event
    pub bg_override: Option<[f32; 4]>,
    pub transparent: bool,
}

impl Settings {
    pub fn new(transparent: bool) -> Self {
        Self {
            transparent,
            ..Self::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cursor_speed: 1.,
            scroll_speed: 1.,
            render_size: None,
            render_target: None,
            bg_override: None,
            transparent: false,
        }
    }
}

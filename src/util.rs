#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vec2 {
    pub x: u64,
    pub y: u64,
}

impl Vec2 {
    pub fn new(x: u64, y: u64) -> Self {
        Self { x, y }
    }
}

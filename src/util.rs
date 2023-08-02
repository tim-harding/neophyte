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

impl Into<(u64, u64)> for Vec2 {
    fn into(self) -> (u64, u64) {
        (self.x, self.y)
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

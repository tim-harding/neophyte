use std::ops::Add;

pub type Vec2 = Vec2T<u64>;
pub type Vec2f = Vec2T<f64>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vec2T<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2T<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Into<(T, T)> for Vec2T<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T: Add<Output = T>> Add for Vec2T<T> {
    type Output = Vec2T<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

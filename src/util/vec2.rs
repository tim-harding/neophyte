use std::ops::{Add, Mul, Sub};

pub type Vec2u = Vec2<u64>;
pub type Vec2s = Vec2<usize>;
pub type Vec2i = Vec2<i64>;
pub type Vec2f = Vec2<f64>;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Into<(T, T)> for Vec2<T> {
    fn into(self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T> Add for Vec2<T>
where
    T: Add<Output = T>,
{
    type Output = Vec2<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Sub for Vec2<T>
where
    T: Sub<Output = T>,
{
    type Output = Vec2<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Mul for Vec2<T>
where
    T: Mul<Output = T>,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

macro_rules! vec_try_from {
    ($t:ty, $u:ty) => {
        impl TryFrom<Vec2<$t>> for Vec2<$u>
        where
            $u: TryFrom<$t>,
        {
            type Error = <$u as TryFrom<$t>>::Error;

            fn try_from(value: Vec2<$t>) -> Result<Self, Self::Error> {
                Ok(Vec2::new(value.x.try_into()?, value.y.try_into()?))
            }
        }
    };
}

vec_try_from!(u64, i64);
vec_try_from!(i64, u64);
vec_try_from!(u64, usize);
vec_try_from!(usize, u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("Failed to convert between vector types")]
pub struct Vec2ConversionError;

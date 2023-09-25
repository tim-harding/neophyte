use bytemuck::{Pod, Zeroable};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

unsafe impl<T> Pod for Vec2<T> where T: Pod {}
unsafe impl<T> Zeroable for Vec2<T> where T: Zeroable {}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Vec2<T>
where
    T: Mul<Output = T> + Clone,
{
    pub fn area(&self) -> T {
        self.x.clone() * self.y.clone()
    }
}

impl<T> From<PhysicalSize<T>> for Vec2<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl<T> From<Vec2<T>> for PhysicalSize<T> {
    fn from(value: Vec2<T>) -> Self {
        Self::new(value.x, value.y)
    }
}

impl<T> From<Vec2<T>> for (T, T) {
    fn from(val: Vec2<T>) -> Self {
        (val.x, val.y)
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from(value: (T, T)) -> Self {
        let (x, y) = value;
        Self::new(x, y)
    }
}

impl<T> From<Vec2<T>> for [T; 2] {
    fn from(value: Vec2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> From<[T; 2]> for Vec2<T> {
    fn from(value: [T; 2]) -> Self {
        let [x, y] = value;
        Self::new(x, y)
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

impl<T> AddAssign for Vec2<T>
where
    T: AddAssign,
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
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

impl<T> SubAssign for Vec2<T>
where
    T: SubAssign,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
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

impl<T> Mul<T> for Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl<T> MulAssign for Vec2<T>
where
    T: MulAssign,
{
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<T> MulAssign<T> for Vec2<T>
where
    T: MulAssign<T> + Copy,
{
    fn mul_assign(&mut self, rhs: T) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl<T> Div for Vec2<T>
where
    T: Div<Output = T>,
{
    type Output = Vec2<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<T> Div<T> for Vec2<T>
where
    T: Div<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl<T> DivAssign for Vec2<T>
where
    T: DivAssign,
{
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<T> DivAssign<T> for Vec2<T>
where
    T: DivAssign<T> + Copy,
{
    fn div_assign(&mut self, rhs: T) {
        self.x /= rhs;
        self.y /= rhs;
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

macro_rules! vec_from {
    ($t:ty, $u:ty) => {
        impl From<Vec2<$t>> for Vec2<$u> {
            fn from(value: Vec2<$t>) -> Self {
                Vec2::new(value.x as $u, value.y as $u)
            }
        }
    };
}

vec_try_from!(u64, i64);
vec_try_from!(i64, u64);
vec_try_from!(u32, i32);
vec_try_from!(i32, u32);
vec_try_from!(u16, i16);
vec_try_from!(i16, u16);
vec_try_from!(u8, i8);
vec_try_from!(i8, u8);
vec_try_from!(u64, usize);
vec_try_from!(usize, u64);
vec_try_from!(i64, isize);
vec_try_from!(isize, i64);
vec_try_from!(u64, u32);
vec_try_from!(u64, u16);
vec_try_from!(u64, u8);
vec_try_from!(i64, i32);
vec_try_from!(i64, i16);
vec_try_from!(i64, i8);

vec_from!(u32, u64);
vec_from!(u16, u64);
vec_from!(u8, u64);

vec_from!(i32, i64);
vec_from!(i16, i64);
vec_from!(i8, i64);

vec_from!(u64, f64);
vec_from!(u32, f64);
vec_from!(u16, f64);
vec_from!(u8, f64);

vec_from!(u64, f32);
vec_from!(u32, f32);
vec_from!(u16, f32);
vec_from!(u8, f32);

vec_from!(f64, f32);
vec_from!(f32, f64);

vec_from!(f64, i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
#[error("Failed to convert between vector types")]
pub struct Vec2ConversionError;

use bytemuck::{Pod, Zeroable};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use winit::dpi::{PhysicalPosition, PhysicalSize};

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

    pub fn map(self, f: fn(T) -> T) -> Self {
        Self::new(f(self.x), f(self.y))
    }
}

impl<T> Vec2<T> where T: Copy {}

impl<T> Vec2<T>
where
    T: Mul<Output = T> + Copy,
{
    pub fn area(&self) -> T {
        self.x * self.y
    }
}

impl<T> Vec2<T>
where
    T: Mul<Output = T> + Add<Output = T> + Copy,
{
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y
    }
}

impl Vec2<f32> {
    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.length()
    }
}

impl Vec2<f64> {
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        self / self.length()
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

impl<T> From<PhysicalPosition<T>> for Vec2<T> {
    fn from(value: PhysicalPosition<T>) -> Self {
        Self::new(value.x, value.y)
    }
}

impl<T> From<Vec2<T>> for PhysicalPosition<T> {
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

impl From<wgpu::Extent3d> for Vec2<u32> {
    fn from(value: wgpu::Extent3d) -> Self {
        Self::new(value.width, value.height)
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
                Vec2::new(value.x.into(), value.y.into())
            }
        }
    };
}

macro_rules! vec_from_lossy {
    ($t:ty, $u:ty) => {
        impl FromLossy<Vec2<$t>> for Vec2<$u> {
            fn from_lossy(value: Vec2<$t>) -> Self {
                Vec2::new(value.x as $u, value.y as $u)
            }
        }
    };
}

pub trait FromLossy<T> {
    fn from_lossy(t: T) -> Self;
}

pub trait IntoLossy<T> {
    fn into_lossy(self) -> T;
}

impl<A, B> IntoLossy<B> for A
where
    B: FromLossy<A>,
{
    fn into_lossy(self) -> B {
        B::from_lossy(self)
    }
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
vec_from!(u16, u32);
vec_from!(u8, u32);
vec_from!(u8, u16);

vec_from!(i32, i64);
vec_from!(i16, i64);
vec_from!(i8, i64);
vec_from!(u32, i64);
vec_from!(u16, i64);
vec_from!(u8, i64);

vec_from!(i16, i32);
vec_from!(i8, i32);
vec_from!(u16, i32);
vec_from!(u8, i32);

vec_from!(i8, i16);
vec_from!(u8, i16);

vec_from_lossy!(u64, f64);
vec_from!(u32, f64);
vec_from!(u16, f64);
vec_from!(u8, f64);

vec_from_lossy!(u64, f32);
vec_from_lossy!(u32, f32);
vec_from!(u16, f32);
vec_from!(u8, f32);

vec_from_lossy!(i64, f64);
vec_from!(i32, f64);
vec_from!(i16, f64);
vec_from!(i8, f64);

vec_from_lossy!(i64, f32);
vec_from_lossy!(i32, f32);
vec_from!(i16, f32);
vec_from!(i8, f32);

vec_from_lossy!(f64, f32);
vec_from!(f32, f64);

vec_from_lossy!(f64, i64);
vec_from_lossy!(f64, i32);
vec_from_lossy!(f64, i16);
vec_from_lossy!(f64, i8);

vec_from_lossy!(f32, i64);
vec_from_lossy!(f32, i32);
vec_from_lossy!(f32, i16);
vec_from_lossy!(f32, i8);

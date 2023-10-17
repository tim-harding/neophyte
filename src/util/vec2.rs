use bytemuck::{Pod, Zeroable};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use winit::dpi::{PhysicalPosition, PhysicalSize};

/// A 2D vector type
#[repr(C, align(8))]
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

    /// Apply f to x and y
    pub fn map(self, f: fn(T) -> T) -> Self {
        Self::new(f(self.x), f(self.y))
    }

    /// Combines lhs and rhs with { x: f(lhs.x, rhs.x), y: f(lhs.y, rhs.y) }
    pub fn combine(lhs: Self, rhs: Self, f: fn(T, T) -> T) -> Self {
        Self::new(f(lhs.x, rhs.x), f(lhs.y, rhs.y))
    }

    /// Swaps the x and y components
    pub fn transpose(self) -> Self {
        Self {
            x: self.y,
            y: self.x,
        }
    }

    /// Equivalent to Into for converting between vectors
    ///
    /// We don't use Into because the implementation conflicts with blanket
    /// impls from the standard library.
    pub fn cast<F>(self) -> Vec2<F>
    where
        F: From<T>,
    {
        Vec2 {
            x: F::from(self.x),
            y: F::from(self.y),
        }
    }

    /// Equivalent to TryInto for converting between vectors
    ///
    /// We don't use TryInto because the implementation conflicts with blanket
    /// impls from the standard library.
    pub fn try_cast<F>(self) -> Result<Vec2<F>, <F as TryFrom<T>>::Error>
    where
        F: TryFrom<T>,
    {
        Ok(Vec2 {
            x: F::try_from(self.x)?,
            y: F::try_from(self.y)?,
        })
    }

    /// Uses the as operator to convert to the destination generic parameter.
    /// Useful when cast and try_cast are not available, such for saturating
    /// conversions between integer and floating point types.
    pub fn cast_as<F>(self) -> Vec2<F>
    where
        T: As<F>,
    {
        Vec2 {
            x: self.x.r#as(),
            y: self.y.r#as(),
        }
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

macro_rules! float_impl {
    ($t:ty) => {
        impl Vec2<$t> {
            pub fn length(&self) -> $t {
                self.length_squared().sqrt()
            }

            pub fn normalized(self) -> Self {
                self / self.length()
            }

            pub fn lerp(self, other: Self, t: $t) -> Self {
                let t = t.max(0.0).min(1.0);
                self * (1.0 - t) + other * t
            }
        }
    };
}

float_impl!(f32);
float_impl!(f64);

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

impl<T> Neg for Vec2<T>
where
    T: Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

pub trait As<T> {
    fn r#as(self) -> T;
}

macro_rules! as_impl {
    ($t:ty, $a:ty) => {
        impl As<$a> for $t {
            fn r#as(self) -> $a {
                self as $a
            }
        }
    };
}

macro_rules! as_impls {
    ($t:ty) => {
        as_impl!($t, u8);
        as_impl!($t, u16);
        as_impl!($t, u32);
        as_impl!($t, u64);
        as_impl!($t, u128);
        as_impl!($t, i8);
        as_impl!($t, i16);
        as_impl!($t, i32);
        as_impl!($t, i64);
        as_impl!($t, i128);
        as_impl!($t, f32);
        as_impl!($t, f64);
    };
}

as_impls!(u8);
as_impls!(u16);
as_impls!(u32);
as_impls!(u64);
as_impls!(u128);
as_impls!(i8);
as_impls!(i16);
as_impls!(i32);
as_impls!(i64);
as_impls!(i128);
as_impls!(f32);
as_impls!(f64);

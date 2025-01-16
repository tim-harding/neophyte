use bytemuck::{Pod, Zeroable};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use wgpu_types::Extent3d;
use winit::dpi::{PhysicalPosition, PhysicalSize};

/// A 2D vector type
// Align is useful to make sure padding is handled correctly in push constant
// structs
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

unsafe impl<T> Pod for Vec2<T> where T: Pod {}
unsafe impl<T> Zeroable for Vec2<T> where T: Zeroable {}

impl<T> Vec2<T>
where
    T: Copy,
{
    pub fn splat(xy: T) -> Self {
        Self::new(xy, xy)
    }
}

impl<T> IntoIterator for Vec2<T> {
    fn into_iter(self) -> Self::IntoIter {
        [self.x, self.y].into_iter()
    }

    type Item = T;

    type IntoIter = std::array::IntoIter<Self::Item, 2>;
}

impl<T> Vec2<T> {
    pub const fn new(x: T, y: T) -> Self {
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

impl<T> From<PhysicalSize<T>> for PixelVec<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Self::new(value.width, value.height)
    }
}

impl<T> From<PixelVec<T>> for PhysicalSize<T> {
    fn from(value: PixelVec<T>) -> Self {
        Self::new(value.0.x, value.0.y)
    }
}

impl<T> From<PhysicalPosition<T>> for PixelVec<T> {
    fn from(value: PhysicalPosition<T>) -> Self {
        Self::new(value.x, value.y)
    }
}

impl<T> From<PhysicalPosition<T>> for Vec2<T> {
    fn from(value: PhysicalPosition<T>) -> Self {
        Self::new(value.x, value.y)
    }
}

impl<T> From<PixelVec<T>> for PhysicalPosition<T> {
    fn from(value: PixelVec<T>) -> Self {
        Self::new(value.0.x, value.0.y)
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

impl From<Extent3d> for PixelVec<u32> {
    fn from(value: Extent3d) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<PixelVec<u32>> for Extent3d {
    fn from(value: PixelVec<u32>) -> Self {
        Self {
            width: value.0.x,
            height: value.0.y,
            depth_or_array_layers: 1,
        }
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

impl<T> Add<T> for Vec2<T>
where
    T: Add<Output = T> + Copy,
{
    type Output = Vec2<T>;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
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

impl<T> AddAssign<T> for Vec2<T>
where
    T: AddAssign<T> + Copy,
{
    fn add_assign(&mut self, rhs: T) {
        self.x += rhs;
        self.y += rhs;
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

impl<T> Sub<T> for Vec2<T>
where
    T: Sub<Output = T> + Copy,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
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

impl<T> SubAssign<T> for Vec2<T>
where
    T: SubAssign<T> + Copy,
{
    fn sub_assign(&mut self, rhs: T) {
        self.x -= rhs;
        self.y -= rhs;
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
        as_impl!($t, isize);
        as_impl!($t, usize);
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
as_impls!(isize);
as_impls!(usize);
as_impls!(f32);
as_impls!(f64);

// TODO: Instead of macros, create a Vec2Newtype trait and create impls for it
macro_rules! newtype_impls {
    ($t:ident) => {
        impl<T> Add for $t<T>
        where
            T: Add<Output = T>,
        {
            type Output = $t<T>;

            fn add(self, rhs: Self) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl<T> AddAssign for $t<T>
        where
            T: AddAssign,
        {
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl<T> Add<T> for $t<T>
        where
            T: Add<Output = T> + Copy,
        {
            type Output = $t<T>;

            fn add(self, rhs: T) -> Self::Output {
                Self(self.0 + rhs)
            }
        }

        impl<T> AddAssign<T> for $t<T>
        where
            T: AddAssign + Copy,
        {
            fn add_assign(&mut self, rhs: T) {
                self.0 += rhs;
            }
        }

        impl<T> Sub for $t<T>
        where
            T: Sub<Output = T>,
        {
            type Output = $t<T>;

            fn sub(self, rhs: Self) -> Self::Output {
                Self(self.0 - rhs.0)
            }
        }

        impl<T> SubAssign for $t<T>
        where
            T: SubAssign,
        {
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl<T> Sub<T> for $t<T>
        where
            T: Sub<Output = T> + Copy,
        {
            type Output = $t<T>;

            fn sub(self, rhs: T) -> Self::Output {
                Self(self.0 - rhs)
            }
        }

        impl<T> SubAssign<T> for $t<T>
        where
            T: SubAssign + Copy,
        {
            fn sub_assign(&mut self, rhs: T) {
                self.0 -= rhs;
            }
        }

        impl<T> Mul for $t<T>
        where
            T: Mul<Output = T>,
        {
            type Output = $t<T>;

            fn mul(self, rhs: Self) -> Self::Output {
                Self(self.0 * rhs.0)
            }
        }

        impl<T> MulAssign for $t<T>
        where
            T: MulAssign,
        {
            fn mul_assign(&mut self, rhs: Self) {
                self.0 *= rhs.0;
            }
        }

        impl<T> Mul<T> for $t<T>
        where
            T: Mul<Output = T> + Copy,
        {
            type Output = $t<T>;

            fn mul(self, rhs: T) -> Self::Output {
                Self(self.0 * rhs)
            }
        }

        impl<T> MulAssign<T> for $t<T>
        where
            T: MulAssign + Copy,
        {
            fn mul_assign(&mut self, rhs: T) {
                self.0 *= rhs;
            }
        }

        impl<T> Div for $t<T>
        where
            T: Div<Output = T>,
        {
            type Output = $t<T>;

            fn div(self, rhs: Self) -> Self::Output {
                Self(self.0 / rhs.0)
            }
        }

        impl<T> DivAssign for $t<T>
        where
            T: DivAssign,
        {
            fn div_assign(&mut self, rhs: Self) {
                self.0 /= rhs.0;
            }
        }

        impl<T> Div<T> for $t<T>
        where
            T: Div<Output = T> + Copy,
        {
            type Output = $t<T>;

            fn div(self, rhs: T) -> Self::Output {
                Self(self.0 / rhs)
            }
        }

        impl<T> DivAssign<T> for $t<T>
        where
            T: DivAssign + Copy,
        {
            fn div_assign(&mut self, rhs: T) {
                self.0 /= rhs;
            }
        }

        impl<T> Neg for $t<T>
        where
            T: Neg<Output = T>,
        {
            type Output = $t<T>;

            fn neg(self) -> Self::Output {
                Self(-self.0)
            }
        }

        impl<T> $t<T>
        where
            T: Copy,
        {
            pub fn splat(xy: T) -> Self {
                Self(Vec2::splat(xy))
            }
        }

        impl<T> $t<T> {
            /// Equivalent to Into for converting between vectors
            ///
            /// We don't use Into because the implementation conflicts with blanket
            /// impls from the standard library.
            pub fn cast<F>(self) -> $t<F>
            where
                F: From<T>,
            {
                $t(self.0.cast())
            }

            /// Equivalent to TryInto for converting between vectors
            ///
            /// We don't use TryInto because the implementation conflicts with blanket
            /// impls from the standard library.
            pub fn try_cast<F>(self) -> Result<$t<F>, <F as TryFrom<T>>::Error>
            where
                F: TryFrom<T>,
            {
                Ok($t(self.0.try_cast()?))
            }

            /// Uses the as operator to convert to the destination generic parameter.
            /// Useful when cast and try_cast are not available, such for saturating
            /// conversions between integer and floating point types.
            pub fn cast_as<F>(self) -> $t<F>
            where
                T: As<F>,
            {
                $t(self.0.cast_as())
            }

            pub fn new(x: T, y: T) -> Self {
                Self(Vec2::new(x, y))
            }

            /// Apply f to x and y
            pub fn map(self, f: fn(T) -> T) -> Self {
                Self(self.0.map(f))
            }

            /// Combines lhs and rhs with { x: f(lhs.x, rhs.x), y: f(lhs.y, rhs.y) }
            pub fn combine(lhs: Self, rhs: Self, f: fn(T, T) -> T) -> Self {
                Self(Vec2::combine(lhs.0, rhs.0, f))
            }

            /// Swaps the x and y components
            pub fn transpose(self) -> Self {
                Self(self.0.transpose())
            }
        }

        impl<T> $t<T>
        where
            T: Mul<Output = T> + Copy,
        {
            pub fn area(&self) -> T {
                self.0.area()
            }
        }

        impl<T> $t<T>
        where
            T: Mul<Output = T> + Add<Output = T> + Copy,
        {
            pub fn length_squared(&self) -> T {
                self.0.length_squared()
            }
        }

        macro_rules! newtype_float_impl {
            ($f:ty) => {
                impl $t<$f> {
                    pub fn length(&self) -> $f {
                        self.0.length()
                    }

                    pub fn normalized(self) -> Self {
                        Self(self.0 / self.length())
                    }

                    pub fn lerp(self, other: Self, t: $f) -> Self {
                        Self(self.0.lerp(other.0, t))
                    }
                }
            };
        }

        newtype_float_impl!(f32);
        newtype_float_impl!(f64);
    };
}

/// A [Vec2] in units of pixels
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Pod, Zeroable)]
pub struct PixelVec<T>(pub Vec2<T>);

impl<T> PixelVec<T>
where
    T: Div<Output = T>,
{
    pub fn into_cells(self, cell_size: Vec2<T>) -> CellVec<T> {
        CellVec(self.0 / cell_size)
    }
}

/// A [Vec2] in units of grid cells
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Pod, Zeroable)]
pub struct CellVec<T>(pub Vec2<T>);

impl<T> CellVec<T>
where
    T: Mul<Output = T>,
{
    pub fn into_pixels(self, cell_size: Vec2<T>) -> PixelVec<T> {
        PixelVec(self.0 * cell_size)
    }
}

impl CellVec<f32> {
    pub fn round_to_pixels(self, cell_size: Vec2<u32>) -> PixelVec<i32> {
        PixelVec((self.0 * cell_size.cast_as()).cast_as())
    }
}

newtype_impls!(PixelVec);
newtype_impls!(CellVec);

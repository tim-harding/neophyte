use super::vec2::Vec2;
use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Mul};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Mat3 {
    // Columns
    x: Vec3,
    y: Vec3,
    z: Vec3,
}

impl Mat3 {
    pub const IDENTITY: Self = Self::with_columns(Vec3::X, Vec3::Y, Vec3::Z);

    pub const fn with_columns(x: Vec3, y: Vec3, z: Vec3) -> Self {
        Self { x, y, z }
    }

    pub fn rotate(radians: f32) -> Self {
        let sin = radians.sin();
        let cos = radians.cos();
        Self {
            x: Vec3::new(cos, sin, 0.0),
            y: Vec3::new(-sin, cos, 0.0),
            z: Vec3::Z,
        }
    }

    pub fn translate(axes: Vec2<f32>) -> Self {
        Self {
            x: Vec3::X,
            y: Vec3::Y,
            z: axes.into(),
        }
    }

    pub fn scale(axes: Vec2<f32>) -> Self {
        Self {
            x: Vec3::X * axes.x,
            y: Vec3::Y * axes.y,
            z: Vec3::Z,
        }
    }
}

impl Mul<Vec3> for Mat3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul for Mat3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
    padding: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self::splat(0.0);
    pub const ONE: Self = Self::splat(1.0);
    pub const X: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z: Self = Self::new(0.0, 0.0, 1.0);

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            padding: 0.0,
        }
    }

    pub const fn splat(n: f32) -> Self {
        Self::new(n, n, n)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            padding: 0.0,
        }
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            padding: 0.0,
        }
    }
}

impl From<Vec2<f32>> for Vec3 {
    fn from(value: Vec2<f32>) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: 1.0,
            padding: 0.0,
        }
    }
}

impl From<Vec3> for Vec2<f32> {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x / value.z,
            y: value.y / value.z,
        }
    }
}

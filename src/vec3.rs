use std::ops::{Add, Div, Mul, Neg};

pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Debug, Default, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn length(&self) -> f32 {
        f32::sqrt(self.x.sqrt() + self.y.sqrt() + self.z.sqrt())
    }

    pub fn length_squared(&self) -> f32 {
        self.x.sqrt() + self.y.sqrt() + self.z.sqrt()
    }

    pub fn dot(&self, rhs: &Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(&self, rhs: &Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
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
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, value: f32) -> Self::Output {
        Self {
            x: self.x * value,
            y: self.y * value,
            z: self.z * value,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, value: f32) -> Self::Output {
        Self {
            x: self.x / value,
            y: self.y / value,
            z: self.z / value,
        }
    }
}

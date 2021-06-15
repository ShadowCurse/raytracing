use rand::distributions::Distribution;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};

pub type Point3 = Vec3;
pub type Color = Vec3;

#[derive(Debug, Default, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn random(min: f32, max: f32) -> Self {
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(min, max);
        Self {
            x: uniform.sample(&mut rng),
            y: uniform.sample(&mut rng),
            z: uniform.sample(&mut rng),
        }
    }

    pub fn random_vec2(min: f32, max: f32) -> Self {
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(min, max);
        Self {
            x: uniform.sample(&mut rng),
            y: uniform.sample(&mut rng),
            z: 0.0,
        }
    }

    pub fn random_unit() -> Self {
        Self::random(-1.0, 1.0).unit()
    }

    pub fn random_in_unit_sphere() -> Self {
        loop {
            let point = Vec3::random(-1.0, 1.0);
            if point.length_squared() < 1.0 {
                return point;
            }
        }
    }

    pub fn random_in_unit_disk() -> Self {
        loop {
            let point = Vec3::random_vec2(-1.0, 1.0);
            if point.length_squared() < 1.0 {
                return point;
            }
        }
    }

    pub fn random_cosine_direction() -> Self {
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::<f32>::new(0.0, 1.0);
        let r1 = uniform.sample(&mut rng);
        let r2 = uniform.sample(&mut rng);
        let z = (1.0 - r2).sqrt();

        let phi = 2.0 * std::f32::consts::PI * r1;
        let x = phi.cos() * r2.sqrt();
        let y = phi.sin() * r2.sqrt();

        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f32 {
        self.x.powi(2) + self.y.powi(2) + self.z.powi(2)
    }

    pub fn near_zero(&self) -> bool {
        let s = 1e-8;
        (self.x < s) && (self.y < s) && (self.z < s)
    }

    pub fn unit(self) -> Self {
        let length = self.length();
        self / length
    }

    pub fn reflect(self, normal: &Self) -> Self {
        self - 2.0 * self.dot(normal) * normal
    }

    pub fn refract(self, normal: &Vec3, etai_over_etat: f32) -> Self {
        let cos_theta = (-self).dot(normal).min(1.0);
        let r_out_perp = etai_over_etat * (self + cos_theta * normal);
        let r_out_parallel = -((1.0 - r_out_perp.length_squared()).abs().sqrt()) * normal;
        r_out_perp + r_out_parallel
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

impl Add<Vec3> for &Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<&Vec3> for &Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &Vec3) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, value: Vec3) -> Self::Output {
        Self::Output {
            x: value.x * self,
            y: value.y * self,
            z: value.z * self,
        }
    }
}

impl Mul<&Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, value: &Vec3) -> Self::Output {
        Self::Output {
            x: value.x * self,
            y: value.y * self,
            z: value.z * self,
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

impl Mul<bool> for Vec3 {
    type Output = Self;

    fn mul(self, value: bool) -> Self {
        return if value { self } else { -self };
    }
}

impl Mul<bool> for &Vec3 {
    type Output = Vec3;

    fn mul(self, value: bool) -> Self::Output {
        return if value { *self } else { -*self };
    }
}

impl Mul<Vec3> for bool {
    type Output = Vec3;

    fn mul(self, value: Vec3) -> Self::Output {
        return if self { value } else { -value };
    }
}

impl Mul<&Vec3> for bool {
    type Output = Vec3;

    fn mul(self, value: &Vec3) -> Self::Output {
        return if self { *value } else { -*value };
    }
}

impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, value: Self) -> Self::Output {
        Self {
            x: self.x * value.x,
            y: self.y * value.y,
            z: self.z * value.z,
        }
    }
}

impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl MulAssign<&Vec3> for Vec3 {
    fn mul_assign(&mut self, rhs: &Vec3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
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

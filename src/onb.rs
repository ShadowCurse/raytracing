use crate::vec3::Vec3;

pub struct Onb {
    pub u: Vec3,
    pub v: Vec3,
    pub w: Vec3,
}

impl Onb {
    pub fn new_from_w(n: &Vec3) -> Self {
        let w = n.unit();
        let a = if w.x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v = w.cross(&a).unit();
        let u = w.cross(&v);
        Self { u, v, w }
    }

    pub fn local_from_points(&self, a: f32, b: f32, c: f32) -> Vec3 {
        a * self.u + b * self.v + c * self.w
    }

    pub fn local_from_vec(&self, vec: &Vec3) -> Vec3 {
        vec.x * self.u + vec.y * self.v + vec.z * self.w
    }
}

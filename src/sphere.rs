use crate::hittable::*;
use crate::material::*;
use crate::ray::*;
use crate::vec3::*;
use std::rc::Rc;

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, material: Rc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let os = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = os.dot(&ray.direction);
        let c = os.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrtd = discriminant.sqrt();

        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;
        let mut record = HitRecord {
            point,
            t: root,
            material: Some(self.material.clone()),
            ..Default::default()
        };
        record.set_face_normal(&ray, &outward_normal);
        Some(record)
    }
}

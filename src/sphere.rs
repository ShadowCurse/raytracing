use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::material::WithMaterialTrait;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

use std::borrow::Borrow;
use std::sync::Arc;

pub struct Sphere {
    pub center: Point3,
    pub radius: f32,
    pub material: Arc<WithMaterialTrait>,
}

fn get_sphere_uv(point: &Point3) -> (f32, f32) {
    // point: a given point on the sphere of radius one, centered atg the origin.
    // (f32, f32) = (u, v): values [0. 1] of angle
    // u: around the Y axis from X = -1
    // v: from Y = -1 to Y = +1
    use std::f32::consts::PI;

    let theta = -point.y.acos();
    let phi = -point.z.atan2(point.x) + PI;

    (phi / (2.0 * PI), theta / PI)
}

impl Sphere {
    pub fn new(center: Point3, radius: f32, material: Arc<WithMaterialTrait>) -> Self {
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
        let c = os.length_squared() - self.radius.powi(2);

        let discriminant = half_b.powi(2) - a * c;
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
        let (u, v) = get_sphere_uv(&outward_normal);
        Some(HitRecord::new(
            point,
            root,
            u,
            v,
            self.material.borrow(),
            &ray,
            &outward_normal,
        ))
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            self.center - Vec3::new(self.radius, self.radius, self.radius),
            self.center + Vec3::new(self.radius, self.radius, self.radius),
        ))
    }
}

pub struct MovingSphere {
    pub center0: Point3,
    pub center1: Point3,
    pub time0: f32,
    pub time1: f32,
    pub radius: f32,
    pub material: Arc<WithMaterialTrait>,
}

impl MovingSphere {
    pub fn new(
        center0: Point3,
        center1: Point3,
        time0: f32,
        time1: f32,
        radius: f32,
        material: Arc<WithMaterialTrait>,
    ) -> Self {
        Self {
            center0,
            center1,
            time0,
            time1,
            radius,
            material,
        }
    }

    pub fn center(&self, time: f32) -> Point3 {
        self.center0
            + ((time - self.time0) / (self.time1 - self.time0)) * (self.center1 - self.center0)
    }
}
impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let os = ray.origin - self.center(ray.time);
        let a = ray.direction.length_squared();
        let half_b = os.dot(&ray.direction);
        let c = os.length_squared() - self.radius.powi(2);

        let discriminant = half_b.powi(2) - a * c;
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
        let outward_normal = (point - self.center(ray.time)) / self.radius;
        let (u, v) = get_sphere_uv(&outward_normal);
        Some(HitRecord::new(
            point,
            root,
            u,
            v,
            self.material.borrow(),
            &ray,
            &outward_normal,
        ))
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        let r = Vec3::new(self.radius, self.radius, self.radius);
        let box0 = AABB::new(self.center(time0) - r, self.center(time0) + r);
        let box1 = AABB::new(self.center(time1) - r, self.center(time1) + r);
        Some(AABB::surrounding_box(box0, box1))
    }
}

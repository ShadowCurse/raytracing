use crate::aabb::AABB;
use crate::material::{ScatterRecord, WithMaterialTrait};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

use std::borrow::Borrow;
use std::sync::Arc;

#[derive(Default)]
pub struct HitRecord<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Option<&'a WithMaterialTrait>,
    pub t: f32,
    pub u: f32,
    pub v: f32,
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new(
        point: Point3,
        t: f32,
        u: f32,
        v: f32,
        material: &'a WithMaterialTrait,
        ray: &Ray,
        outward_normal: &Vec3,
    ) -> Self {
        let front_face = ray.direction.dot(&outward_normal) < 0.0;
        Self {
            point,
            normal: if front_face {
                *outward_normal
            } else {
                -*outward_normal
            },
            material: Some(material),
            t,
            u,
            v,
            front_face,
        }
    }

    pub fn scatter(&self, ray: &Ray) -> Option<ScatterRecord> {
        self.material.unwrap().scatter(ray, &self)
    }
}

pub type WithHittableTrait = dyn Hittable + Send + Sync;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB>;
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f32 {
        0.0
    }
    fn random(&self, origin: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

pub struct Translate {
    pub object: Arc<WithHittableTrait>,
    pub offset: Vec3,
}

impl Translate {
    pub fn new(object: Arc<WithHittableTrait>, offset: Vec3) -> Self {
        Self { object, offset }
    }
}

impl Hittable for Translate {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        return if let Some(mut hit) = self.object.hit(&moved_ray, t_min, t_max) {
            hit.point += self.offset;
            hit.front_face = moved_ray.direction.dot(&hit.normal) < 0.0;
            Some(hit)
        } else {
            None
        };
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        return if let Some(aabb) = self.object.bounding_box(time0, time1) {
            Some(AABB::new(
                aabb.minimum + self.offset,
                aabb.maximum + self.offset,
            ))
        } else {
            None
        };
    }
}

pub struct Rotate {
    pub object: Arc<WithHittableTrait>,
    pub sin_theta: f32,
    pub cos_theta: f32,
    pub aabb: AABB,
}

impl Rotate {
    pub fn new(object: Arc<WithHittableTrait>, angle: f32) -> Self {
        let radians = angle.to_radians();
        let sin = radians.sin();
        let cos = radians.cos();
        let aabb = object.bounding_box(0.0, 1.0).unwrap();

        let mut min = Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Point3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f32 * aabb.maximum.x + (1.0 - i as f32) * aabb.minimum.x;
                    let y = j as f32 * aabb.maximum.y + (1.0 - j as f32) * aabb.minimum.y;
                    let z = k as f32 * aabb.maximum.z + (1.0 - k as f32) * aabb.minimum.z;

                    let newx = cos * x + sin * z;
                    let newz = -sin * x + cos * z;

                    min.x = min.x.min(newx);
                    min.y = min.y.min(y);
                    min.z = min.z.min(newz);

                    max.x = max.x.max(newx);
                    max.y = max.y.max(y);
                    max.z = max.z.max(newz);
                }
            }
        }
        let aabb = AABB::new(min, max);

        Self {
            object,
            sin_theta: sin,
            cos_theta: cos,
            aabb,
        }
    }
}

impl Hittable for Rotate {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;

        origin.x = self.cos_theta * ray.origin.x - self.sin_theta * ray.origin.z;
        origin.z = self.sin_theta * ray.origin.x + self.cos_theta * ray.origin.z;

        direction.x = self.cos_theta * ray.direction.x - self.sin_theta * ray.direction.z;
        direction.z = self.sin_theta * ray.direction.x + self.cos_theta * ray.direction.z;

        let rotated = Ray::new(origin, direction, ray.time);

        if let Some(mut hit) = self.object.hit(&rotated, t_min, t_max) {
            let mut point = hit.point;
            let mut normal = hit.normal;

            point.x = self.cos_theta * hit.point.x + self.sin_theta * hit.point.z;
            point.z = -self.sin_theta * hit.point.x + self.cos_theta * hit.point.z;

            normal.x = self.cos_theta * hit.normal.x + self.sin_theta * hit.normal.z;
            normal.z = -self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.z;

            hit.point = point;
            hit.front_face = rotated.direction.dot(&normal) < 0.0;
            hit.normal = if hit.front_face { normal } else { -normal };
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(self.aabb)
    }
}

pub struct ConstantMedium {
    pub boundary: Arc<WithHittableTrait>,
    pub phase_function: Arc<WithMaterialTrait>,
    pub neg_inv_density: f32,
}

impl ConstantMedium {
    pub fn new(
        boundary: Arc<WithHittableTrait>,
        density: f32,
        material: Arc<WithMaterialTrait>,
    ) -> Self {
        Self {
            boundary,
            phase_function: material,
            neg_inv_density: -1.0 / density,
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return if let Some(mut hit1) = self.boundary.hit(ray, f32::NEG_INFINITY, f32::INFINITY) {
            return if let Some(mut hit2) = self.boundary.hit(ray, hit1.t + 0.0001, f32::INFINITY) {
                if hit1.t < t_min {
                    hit1.t = t_min;
                }
                if hit2.t > t_max {
                    hit2.t = t_max;
                }

                if hit1.t >= hit2.t {
                    return None;
                }

                if hit1.t < 0.0 {
                    hit1.t = 0.0;
                }

                let ray_length = ray.direction.length();
                let distance_inside_boundary = (hit2.t - hit1.t) * ray_length;
                let hit_distance = self.neg_inv_density * rand::random::<f32>().log2();

                if hit_distance > distance_inside_boundary {
                    return None;
                }

                let t = hit1.t + hit_distance / ray_length;
                let point = ray.at(t);
                let normal = Vec3::new(1.0, 0.0, 0.0);

                let mut record = HitRecord::default();
                record.t = t;
                record.point = point;
                record.normal = normal;
                record.front_face = true;
                record.material = Some(self.phase_function.borrow());
                Some(record)
            } else {
                None
            };
        } else {
            None
        };
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
}

pub struct FlipFace {
    pub object: Arc<WithHittableTrait>,
}

impl FlipFace {
    pub fn new(object: Arc<WithHittableTrait>) -> Self {
        Self { object }
    }
}

impl Hittable for FlipFace {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return if let Some(mut hit) = self.object.hit(ray, t_min, t_max) {
            hit.front_face = !hit.front_face;
            Some(hit)
        } else {
            None
        };
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        self.object.bounding_box(time0, time1)
    }
}

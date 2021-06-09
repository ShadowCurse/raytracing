use crate::aabb::AABB;
use crate::material::WithMaterialTrait;
use crate::ray::Ray;
use crate::vec3::{Color, Point3, Vec3};
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
            normal: front_face * outward_normal,
            material: Some(material),
            t,
            u,
            v,
            front_face,
        }
    }

    pub fn scatter(&self, ray: &Ray) -> Option<(Ray, Color)> {
        self.material.unwrap().scatter(ray, &self)
    }
}

pub type WithHittableTrait = dyn Hittable + Send + Sync;

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB>;
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

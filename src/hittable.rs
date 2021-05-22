use crate::material::*;
use crate::ray::*;
use crate::vec3::*;

#[derive(Default)]
pub struct HitRecord<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Option<&'a WithMaterialTrait>,
    pub t: f32,
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new(
        point: Point3,
        t: f32,
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
}

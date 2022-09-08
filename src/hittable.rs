use crate::aabb::AABB;
use crate::material::ScatterRecord;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use crate::Material;

#[derive(Default)]
pub struct HitRecord<'a> {
    pub point: Point3,
    pub normal: Vec3,
    pub material: Option<&'a dyn Material>,
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
        material: &'a dyn Material,
        ray: &Ray,
        outward_normal: &Vec3,
    ) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
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
        self.material?.scatter(ray, self)
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB>;
    fn pdf_value(&self, _origin: &Point3, _direction: &Vec3) -> f32 {
        0.0
    }
    fn random(&self, _origin: &Vec3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }
}

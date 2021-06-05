use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::texture::WithTexture;
use crate::vec3::{Color, Point3, Vec3};

use rand::Rng;
use std::sync::Arc;

pub type WithMaterialTrait = dyn Material + Sync + Send;

pub trait Material: Sync + Send {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<(Ray, Color)> {
        None
    }
    fn emit(&self, _u: f32, _v: f32, _point: &Point3) -> Color {
        Color::default()
    }
}

pub struct Lambertian {
    pub albedo: Arc<WithTexture>,
}

impl Lambertian {
    pub fn new(albedo: Arc<WithTexture>) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)> {
        let mut scatter_direction = hit_record.normal + Vec3::random_unit();
        if scatter_direction.near_zero() {
            scatter_direction = hit_record.normal;
        }
        Some((
            Ray::new(hit_record.point, scatter_direction, ray.time),
            self.albedo
                .color(hit_record.u, hit_record.v, &hit_record.point),
        ))
    }
}

pub struct Metal {
    pub albedo: Arc<WithTexture>,
    pub fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Arc<WithTexture>, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)> {
        let reflected = ray.direction.unit().reflect(&hit_record.normal);
        let scattered = Ray::new(
            hit_record.point,
            reflected + self.fuzz * Vec3::random_unit(),
            ray.time,
        );
        return if scattered.direction.dot(&hit_record.normal) > 0.0 {
            Some((
                scattered,
                self.albedo
                    .color(hit_record.u, hit_record.v, &hit_record.point),
            ))
        } else {
            None
        };
    }
}

pub struct Dielectric {
    index_of_refraction: f32,
}

impl Dielectric {
    pub fn new(index_of_refraction: f32) -> Self {
        Self {
            index_of_refraction,
        }
    }

    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };
        let unit_direction = ray.direction.unit();
        let cos_theta = (-unit_direction).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let direction = if refraction_ratio * sin_theta > 1.0
            || Self::reflectance(cos_theta, refraction_ratio) > rng.gen()
        {
            // can not refract
            unit_direction.reflect(&hit_record.normal)
        } else {
            // can refract
            unit_direction.refract(&hit_record.normal, refraction_ratio)
        };
        Some((
            Ray::new(hit_record.point, direction, ray.time),
            Color::new(1.0, 1.0, 1.0),
        ))
    }
}

pub struct DiffuseLight {
    pub emit: Arc<WithTexture>,
}

impl DiffuseLight {
    pub fn new(emit: Arc<WithTexture>) -> Self {
        Self { emit }
    }
}

impl Material for DiffuseLight {
    fn emit(&self, u: f32, v: f32, point: &Point3) -> Color {
        self.emit.color(u, v, point)
    }
}

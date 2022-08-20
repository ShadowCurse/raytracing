use std::sync::Arc;

use rand::Rng;

use crate::hittable::HitRecord;
use crate::pdf::{CosinePdf, Pdf};
use crate::ray::Ray;
use crate::texture::WithTexture;
use crate::vec3::{Color, Point3, Vec3};

#[derive(Default)]
pub struct ScatterRecord {
    pub specular_ray: Ray,
    pub is_specular: bool,
    pub attenuation: Color,
    pub pdf: Option<Box<dyn Pdf>>,
}

pub type WithMaterialTrait = dyn Material + Sync + Send;

pub trait Material: Sync + Send {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f32 {
        0.0
    }
    fn emit(&self, _ray: &Ray, _hit: &HitRecord, _u: f32, _v: f32, _point: &Point3) -> Color {
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
    fn scatter(&self, _: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            is_specular: false,
            attenuation: self
                .albedo
                .color(hit_record.u, hit_record.v, &hit_record.point),
            pdf: Some(Box::new(CosinePdf::new(&hit_record.normal))),
            ..Default::default()
        })
    }
    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit_record.normal.dot(&scattered.direction.unit());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
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
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let reflected = ray.direction.unit().reflect(&hit_record.normal);
        Some(ScatterRecord {
            specular_ray: Ray::new(
                hit_record.point,
                reflected + self.fuzz * Vec3::random_in_unit_sphere(),
                ray.time,
            ),
            attenuation: self
                .albedo
                .color(hit_record.u, hit_record.v, &hit_record.point),
            is_specular: true,
            ..Default::default()
        })
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
    fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let refraction_ratio = if hit_record.front_face {
            1.0 / self.index_of_refraction
        } else {
            self.index_of_refraction
        };
        let unit_direction = ray.direction.unit();
        let cos_theta = (-unit_direction).dot(&hit_record.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let direction = if refraction_ratio * sin_theta > 1.0
            || Self::reflectance(cos_theta, refraction_ratio) > rand::thread_rng().gen()
        {
            // can not refract
            unit_direction.reflect(&hit_record.normal)
        } else {
            // can refract
            unit_direction.refract(&hit_record.normal, refraction_ratio)
        };
        Some(ScatterRecord {
            specular_ray: Ray::new(hit_record.point, direction, ray.time),
            is_specular: true,
            attenuation: Color::new(1.0, 1.0, 1.0),
            ..Default::default()
        })
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
    fn emit(&self, _ray: &Ray, hit: &HitRecord, u: f32, v: f32, point: &Point3) -> Color {
        if hit.front_face {
            self.emit.color(u, v, point)
        } else {
            Color::default()
        }
    }
}

pub struct Isotropic {
    pub albedo: Arc<WithTexture>,
}

impl Isotropic {
    pub fn new(texture: Arc<WithTexture>) -> Self {
        Self { albedo: texture }
    }
}

impl Material for Isotropic {
    // fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)> {
    //     Some((
    //         Ray::new(hit_record.point, Vec3::random_in_unit_sphere(), ray.time),
    //         self.albedo
    //             .color(hit_record.u, hit_record.v, &hit_record.point),
    //     ))
    // }
}

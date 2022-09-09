use crate::hittable::HitRecord;
use crate::pdf::{CosinePdf, Pdf};
use crate::ray::Ray;
use crate::vec3::{Color, Point3, Vec3};
use crate::Texture;
use rand::Rng;

#[derive(Default)]
pub struct ScatterRecord {
    pub specular_ray: Ray,
    pub is_specular: bool,
    pub attenuation: Color,
    pub pdf: Option<Box<dyn Pdf>>,
}

pub trait Material {
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Lambertian<T: Texture> {
    pub albedo: T,
}

impl<T: Texture> Lambertian<T> {
    pub fn new(albedo: T) -> Self {
        Self { albedo }
    }
}

impl<T: Texture> Material for Lambertian<T> {
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

#[derive(Default, Debug, Clone, Copy)]
pub struct Metal<T: Texture> {
    pub albedo: T,
    pub fuzz: f32,
}

impl<T: Texture> Metal<T> {
    pub fn new(albedo: T, fuzz: f32) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl<T: Texture> Material for Metal<T> {
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

#[derive(Default, Debug, Clone, Copy)]
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

#[derive(Default, Debug, Clone, Copy)]
pub struct DiffuseLight<T: Texture> {
    pub emit: T,
}

impl<T: Texture> DiffuseLight<T> {
    pub fn new(emit: T) -> Self {
        Self { emit }
    }
}

impl<T: Texture> Material for DiffuseLight<T> {
    fn emit(&self, _ray: &Ray, hit: &HitRecord, u: f32, v: f32, point: &Point3) -> Color {
        if hit.front_face {
            self.emit.color(u, v, point)
        } else {
            Color::default()
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Isotropic<T: Texture> {
    pub albedo: T,
}

impl<T: Texture> Isotropic<T> {
    pub fn new(texture: T) -> Self {
        Self { albedo: texture }
    }
}

impl<T: Texture> Material for Isotropic<T> {
    // fn scatter(&self, ray: &Ray, hit_record: &HitRecord) -> Option<(Ray, Color)> {
    //     Some((
    //         Ray::new(hit_record.point, Vec3::random_in_unit_sphere(), ray.time),
    //         self.albedo
    //             .color(hit_record.u, hit_record.v, &hit_record.point),
    //     ))
    // }
}

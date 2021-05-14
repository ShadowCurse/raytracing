use crate::ray::*;
use crate::vec3::*;

pub trait Material {
    fn scatter(&self, ray_in: &Ray, hit_point: &Point3, hit_normal: &Vec3) -> Option<(Ray, Color)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit_point: &Point3, hit_normal: &Vec3) -> Option<(Ray, Color)> {
        let mut scatter_direction = hit_normal + Vec3::random_unit();
        if scatter_direction.near_zero() {
            scatter_direction = *hit_normal;
        }
        let ray = Ray::new(*hit_point, scatter_direction);
        Some((ray, self.albedo))
    }
}

pub struct Metal {
    pub albedo: Color,
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_point: &Point3, hit_normal: &Vec3) -> Option<(Ray, Color)> {
        let reflected = ray_in.direction.unit().reflect(&hit_normal);
        let scattered = Ray::new(*hit_point, reflected);
        return if scattered.direction.dot(&hit_normal) > 0.0 {
            Some((scattered, self.albedo))
        } else {
            None
        };
    }
}

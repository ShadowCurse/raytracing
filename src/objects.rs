use std::borrow::Borrow;
use std::sync::Arc;

use rand::Rng;

use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::material::WithMaterialTrait;
use crate::onb::Onb;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use crate::world::World;

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

    let theta = (-point.y).acos();
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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f32 {
        if let Some(_) = self.hit(&Ray::new(*origin, *direction, 0.0), 0.001, f32::INFINITY) {
            let cos_theta_max =
                (1.0 - self.radius.powi(2) / (self.center - origin).length_squared()).sqrt();
            let solid_angle = 2.0 * std::f32::consts::PI * (1.0 - cos_theta_max);
            1.0 / solid_angle
        } else {
            0.0
        }
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let dir = self.center - origin;
        let uvw = Onb::new_from_w(&dir);
        uvw.local_from_vec(&Vec3::random_to_sphere(self.radius, dir.length_squared()))
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

pub struct XYRect {
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub k: f32,
    pub material: Arc<WithMaterialTrait>,
}

impl XYRect {
    pub fn new(
        x0: f32,
        x1: f32,
        y0: f32,
        y1: f32,
        k: f32,
        material: Arc<WithMaterialTrait>,
    ) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k,
            material,
        }
    }
}

impl Hittable for XYRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.z) / ray.direction.z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        Some(HitRecord::new(
            ray.at(t),
            t,
            u,
            v,
            self.material.borrow(),
            &ray,
            &Vec3::new(0.0, 0.0, 1.0),
        ))
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.y0, self.k - 0.00001),
            Point3::new(self.x1, self.y1, self.k + 0.00001),
        ))
    }
}

pub struct XZRect {
    pub x0: f32,
    pub x1: f32,
    pub z0: f32,
    pub z1: f32,
    pub k: f32,
    pub material: Arc<WithMaterialTrait>,
}

impl XZRect {
    pub fn new(
        x0: f32,
        x1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        material: Arc<WithMaterialTrait>,
    ) -> Self {
        Self {
            x0,
            x1,
            z0,
            z1,
            k,
            material,
        }
    }
}

impl Hittable for XZRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.y) / ray.direction.y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        Some(HitRecord::new(
            ray.at(t),
            t,
            u,
            v,
            self.material.borrow(),
            &ray,
            &Vec3::new(0.0, 1.0, 0.0),
        ))
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.x0, self.k - 0.00001, self.z0),
            Point3::new(self.x1, self.k + 0.00001, self.z1),
        ))
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f32 {
        if let Some(hit) = self.hit(&Ray::new(*origin, *direction, 0.0), 0.0001, f32::INFINITY) {
            let area = (self.x1 - self.x0) * (self.z1 - self.z0);
            let dist_sqrt = hit.t.powi(2) * direction.length_squared();
            let cosine = (direction.dot(&hit.normal) / direction.length()).abs();
            dist_sqrt / (cosine * area)
        } else {
            0.0
        }
    }
    fn random(&self, origin: &Vec3) -> Vec3 {
        let random_point = Point3::new(
            rand::thread_rng().gen_range(self.x0..self.x1),
            self.k,
            rand::thread_rng().gen_range(self.z0..self.z1),
        );
        random_point - origin
    }
}

pub struct YZRect {
    pub y0: f32,
    pub y1: f32,
    pub z0: f32,
    pub z1: f32,
    pub k: f32,
    pub material: Arc<WithMaterialTrait>,
}

impl YZRect {
    pub fn new(
        y0: f32,
        y1: f32,
        z0: f32,
        z1: f32,
        k: f32,
        material: Arc<WithMaterialTrait>,
    ) -> Self {
        Self {
            y0,
            y1,
            z0,
            z1,
            k,
            material,
        }
    }
}

impl Hittable for YZRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.x) / ray.direction.x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }
        let u = (y - self.y0) / (self.y1 - self.y0);
        let v = (z - self.z0) / (self.z1 - self.z0);
        Some(HitRecord::new(
            ray.at(t),
            t,
            u,
            v,
            self.material.borrow(),
            &ray,
            &Vec3::new(1.0, 0.0, 0.0),
        ))
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(
            Point3::new(self.k - 0.00001, self.y0, self.z0),
            Point3::new(self.k + 0.00001, self.y1, self.z1),
        ))
    }
}

pub struct Box3d {
    pub min: Point3,
    pub max: Point3,
    pub sides: World,
}

impl Box3d {
    pub fn new(min: Point3, max: Point3, material: Arc<WithMaterialTrait>) -> Self {
        let mut world = World::default();

        world.add_object(Arc::new(XYRect::new(
            min.x,
            max.x,
            min.y,
            max.y,
            max.z,
            material.clone(),
        )));
        world.add_object(Arc::new(XYRect::new(
            min.x,
            max.x,
            min.y,
            max.y,
            min.z,
            material.clone(),
        )));

        world.add_object(Arc::new(XZRect::new(
            min.x,
            max.x,
            min.z,
            max.z,
            max.y,
            material.clone(),
        )));
        world.add_object(Arc::new(XZRect::new(
            min.x,
            max.x,
            min.z,
            max.z,
            min.y,
            material.clone(),
        )));

        world.add_object(Arc::new(YZRect::new(
            min.y,
            max.y,
            min.z,
            max.z,
            max.x,
            material.clone(),
        )));
        world.add_object(Arc::new(YZRect::new(
            min.y,
            max.y,
            min.z,
            max.z,
            min.x,
            material.clone(),
        )));

        Self {
            min,
            max,
            sides: world,
        }
    }
}

impl Hittable for Box3d {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.sides.hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(AABB::new(self.min, self.max))
    }
}

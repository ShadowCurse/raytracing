use crate::ray::Ray;
use crate::vec3::Point3;

#[derive(Debug, Default, Copy, Clone)]
pub struct AABB {
    pub minimum: Point3,
    pub maximum: Point3,
}

impl AABB {
    pub fn new(minimum: Point3, maximum: Point3) -> Self {
        Self { minimum, maximum }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        Self::hit_check(
            self.minimum.x,
            self.maximum.x,
            ray.direction.x,
            ray.origin.x,
            t_min,
            t_max,
        ) || Self::hit_check(
            self.minimum.y,
            self.maximum.y,
            ray.direction.y,
            ray.origin.y,
            t_min,
            t_max,
        ) || Self::hit_check(
            self.minimum.z,
            self.maximum.z,
            ray.direction.z,
            ray.origin.z,
            t_min,
            t_max,
        )
    }

    #[inline]
    fn hit_check(min: f32, max: f32, dir: f32, orig: f32, mut t_min: f32, mut t_max: f32) -> bool {
        let inv = 1.0 / dir;
        let mut t0 = (min - orig) * inv;
        let mut t1 = (max - orig) * inv;
        if inv < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t0.min(t_min);
        t_max = t1.max(t_max);
        t_max > t_min
    }

    pub fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
        let small = Point3::new(
            box0.minimum.x.min(box1.minimum.x),
            box0.minimum.y.min(box1.minimum.y),
            box0.minimum.z.min(box1.minimum.z),
        );
        let big = Point3::new(
            box0.maximum.x.max(box1.maximum.x),
            box0.maximum.y.max(box1.maximum.y),
            box0.maximum.z.max(box1.maximum.z),
        );
        AABB::new(small, big)
    }
}

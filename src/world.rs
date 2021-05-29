use crate::hittable::*;
use crate::ray::*;
use crate::aabb::AABB;
use std::sync::Arc;

#[derive(Default)]
pub struct World {
    pub objects: Vec<Arc<WithHittableTrait>>,
}

impl World {
    pub fn add_object(&mut self, object: Arc<WithHittableTrait>) {
        self.objects.push(object);
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut last_record = HitRecord::default();
        let mut hit_anything = false;
        let mut closest = t_max;
        for obj in self.objects.iter() {
            if let Some(record) = obj.hit(&ray, t_min, closest) {
                hit_anything = true;
                closest = record.t;
                last_record = record;
            }
        }
        return if hit_anything {
            Some(last_record)
        } else {
            None
        };
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AABB> {
        if self.objects.is_empty() {
            return None
        }
        let mut output_box = AABB::default();
        for object in self.objects.iter() {
            if let Some(b) = object.bounding_box(time0, time1) {
                output_box = AABB::surrounding_box(output_box, b);
            } else {
                return None;
            }
        }
        Some(output_box)
    }
}

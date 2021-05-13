use crate::hittable::*;
use crate::ray::*;
use crate::vec3::*;
use sdl2::event::WindowEvent::Hidden;

#[derive(Default)]
pub struct World {
    pub objects: Vec<Box<dyn Hittable>>,
}

impl World {
    pub fn add_object(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut last_record = HitRecord::default();
        let mut hit_anything = false;
        let mut closest = t_max;
        for obj in self.objects.iter() {
            if let Some(record) = obj.hit(&ray, t_min, t_max) {
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
}

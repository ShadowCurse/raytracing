use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

use rand::Rng;

use crate::aabb::AABB;
use crate::blobvec::BlobVec;
use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::{HittableVTable, Point3, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct WorldIndex {
    type_index: usize,
    object_index: usize,
}

#[derive(Default)]
pub struct World {
    aabb: AABB,
    types: HashMap<TypeId, usize>,
    data: Vec<(HittableVTable, BlobVec)>,
}

impl World {
    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        let id = TypeId::of::<T>();
        let types = &mut self.types;
        let data = &mut self.data;
        let index = types.entry(id).or_insert_with(|| {
            let vtable = HittableVTable::new::<T>();
            let blob_vec = BlobVec::new(Layout::new::<T>());
            data.push((vtable, blob_vec));
            data.len() - 1
        });
        self.aabb = AABB::surrounding_box(self.aabb, object.bounding_box());
        let (_vtable, blob_vec) = &mut self.data[*index];
        unsafe { blob_vec.add(object) };
    }

    pub fn volumes(&self) -> Vec<(WorldIndex, AABB)> {
        let mut volumes = Vec::new();
        for (_, type_index) in self.types.iter() {
            let (vtable, blob) = &self.data[*type_index];
            for object_index in 0..blob.len() {
                let ptr = unsafe { blob.get(object_index) };
                let aabb = vtable.bounding_box(ptr);
                volumes.push((
                    WorldIndex {
                        type_index: *type_index,
                        object_index,
                    },
                    aabb,
                ));
            }
        }
        volumes
    }

    pub fn hit_object(
        &self,
        index: &WorldIndex,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        let (vtable, blob) = &self.data[index.type_index];
        let ptr = unsafe { blob.get(index.object_index) };
        vtable.hit(ptr, ray, t_min, t_max)
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut last_record = HitRecord::default();
        let mut hit_anything = false;
        let mut closest = t_max;
        for (vtable, blob) in self.data.iter() {
            for i in 0..blob.len() {
                let ptr = unsafe { blob.get(i) };
                if let Some(record) = vtable.hit(ptr, ray, t_min, closest) {
                    hit_anything = true;
                    closest = record.t;
                    last_record = record;
                }
            }
        }
        if hit_anything {
            Some(last_record)
        } else {
            None
        }
    }

    fn bounding_box(&self) -> AABB {
        self.aabb
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f32 {
        let total_objects = self.data.iter().fold(0, |sum, (_, blob)| sum + blob.len());
        let weight = 1.0 / total_objects as f32;

        self.data.iter().fold(0.0, |mut sum, (vtable, blob)| {
            for i in 0..blob.len() {
                sum += weight * vtable.pdf_value(unsafe { blob.get(i) }, origin, direction)
            }
            sum
        })
    }

    fn random(&self, origin: &Vec3) -> Vec3 {
        let obj_type = rand::thread_rng().gen_range(0..self.data.len());
        let (vtable, blob) = &self.data[obj_type];
        let obj_pos = rand::thread_rng().gen_range(0..blob.len());
        vtable.random(unsafe { blob.get(obj_pos) }, origin)
    }
}

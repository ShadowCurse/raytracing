use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};
use std::alloc::Layout;
use std::any::TypeId;
use std::collections::HashMap;

pub struct BlobVec {
    layout: Layout,
    len: usize,
    data: Vec<u8>,
}

impl BlobVec {
    pub fn new(layout: Layout) -> Self {
        Self {
            layout,
            len: 0,
            data: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub unsafe fn add<T>(&mut self, object: T) {
        assert!(
            self.layout == Layout::new::<T>(),
            "adding type with different layout"
        );
        let ptr: &u8 = std::mem::transmute(&object);
        let slice = std::slice::from_raw_parts(ptr, self.layout.size());
        self.data.extend_from_slice(slice);
        self.len += 1;
        std::mem::forget(object);
    }

    pub unsafe fn get(&self, index: usize) -> &() {
        let data_index = index * self.layout.size();
        std::mem::transmute(&self.data[data_index])
    }
}


pub struct HittableVTable {
    pub hit: for<'a, 'b> fn(&'a (), &'b Ray, f32, f32) -> Option<HitRecord<'a>>,
    pub bounding_box: fn(&(), f32, f32) -> Option<AABB>,
    pub pdf_value: fn(&(), &Point3, &Vec3) -> f32,
    pub random: fn(&(), &Vec3) -> Vec3,
}

impl HittableVTable {
    unsafe fn new<T: Hittable>() -> Self {
        Self {
            hit: std::mem::transmute(<T as Hittable>::hit as fn(_, _, _, _) -> _),
            bounding_box: std::mem::transmute(<T as Hittable>::bounding_box as fn(_, _, _) -> _),
            pdf_value: std::mem::transmute(<T as Hittable>::pdf_value as fn(_, _, _) -> _),
            random: std::mem::transmute(<T as Hittable>::random as fn(_, _) -> _),
        }
    }
}

#[derive(Default)]
pub struct ObjectStore {
    aabb: AABB,
    types: HashMap<TypeId, (HittableVTable, BlobVec)>,
}

impl ObjectStore {
    pub fn register_type<T: Hittable + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        let vtable = unsafe { HittableVTable::new::<T>() };
        let blob_vec = BlobVec::new(Layout::new::<T>());
        self.types.insert(id, (vtable, blob_vec));
    }

    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        let id = TypeId::of::<T>();
        assert!(
            self.types.contains_key(&id),
            "type: {:?} should be registered before usage",
            std::any::type_name::<T>()
        );
        self.aabb = AABB::surrounding_box(self.aabb, unsafe {
            object.bounding_box(0.0, 0.0).unwrap_unchecked()
        });
        let (_vtable, blob_vec) = self.types.get_mut(&id).unwrap();
        unsafe { blob_vec.add(object) };
    }

    pub fn hit_object(
        &self,
        id: TypeId,
        index: usize,
        ray: Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        let (vtable, blob_vec) = &self.types[&id];
        let ptr = unsafe { blob_vec.get(index) };
        (vtable.hit)(ptr, &ray, t_min, t_max)
    }
}

impl Hittable for ObjectStore {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut last_record = HitRecord::default();
        let mut hit_anything = false;
        let mut closest = t_max;
        for t in self.types.keys() {
            for i in 0..self.types[t].1.len() {
                if let Some(record) = self.hit_object(*t, i, *ray, t_min, closest) {
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

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<AABB> {
        Some(self.aabb)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blob_new() {
        let layout = Layout::new::<u32>();
        let blob = BlobVec::new(layout);
        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 0);
        assert_eq!(blob.data, []);
    }

    #[test]
    fn blob_add() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.add(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 1);
        assert_eq!(blob.data, [0, 0, 0, 0]);

        let val: u32 = 32;
        unsafe { blob.add(val) };

        assert_eq!(blob.layout, Layout::new::<u32>());
        assert_eq!(blob.len, 2);
        assert_eq!(blob.data, [0, 0, 0, 0, 32, 0, 0, 0]);
    }

    #[test]
    fn blob_get() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.add(val) };

        let ptr = unsafe { blob.get(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;

        assert_eq!(ptr, blob.data.as_ptr());

        let val: u32 = 32;
        unsafe { blob.add(val) };

        let ptr = unsafe { blob.get(0) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, blob.data.as_ptr());
        let ptr = unsafe { blob.get(1) };
        let ptr: *const u8 = ptr as *const () as *const u8;
        assert_eq!(ptr, &blob.data[4] as *const u8);
    }
}

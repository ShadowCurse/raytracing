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

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
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

    #[inline]
    pub unsafe fn get(&self, index: usize) -> &() {
        let data_index = index * self.layout.size();
        std::mem::transmute(&self.data[data_index])
    }

    #[inline]
    pub unsafe fn as_slice<T>(&self) -> &[T] {
        assert!(
            self.layout == Layout::new::<T>(),
            "casting to type with different layout"
        );
        let ptr: *const T = std::mem::transmute(self.data.as_ptr());
        std::slice::from_raw_parts(ptr, self.len)
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
pub struct World {
    aabb: AABB,
    types: HashMap<TypeId, usize>,
    data: Vec<(HittableVTable, BlobVec)>,
}

impl World {

    pub fn add<T: Hittable + 'static>(&mut self, object: T) {
        let id = TypeId::of::<T>();
        if !self.types.contains_key(&id) { 
            let vtable = unsafe { HittableVTable::new::<T>() };
            let blob_vec = BlobVec::new(Layout::new::<T>());
            self.data.push((vtable, blob_vec));
            self.types.insert(id, self.data.len() - 1);
        };
        self.aabb = AABB::surrounding_box(self.aabb, unsafe {
            object.bounding_box(0.0, 0.0).unwrap_unchecked()
        });
        let index = self.types[&id];
        let (_vtable, blob_vec) = &mut self.data[index];
        unsafe { blob_vec.add(object) };
    }

    #[inline]
    pub fn hit_object(
        &self,
        data_index: usize,
        index: usize,
        ray: Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        let (vtable, blob_vec) = &self.data[data_index];
        let ptr = unsafe { blob_vec.get(index) };
        (vtable.hit)(ptr, &ray, t_min, t_max)
    }
}

impl Hittable for World {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut last_record = HitRecord::default();
        let mut hit_anything = false;
        let mut closest = t_max;
        for (di, (_, blob)) in self.data.iter().enumerate() {
            for i in 0..blob.len() {
                if let Some(record) = self.hit_object(di, i, *ray, t_min, closest) {
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

    #[test]
    fn blob_as_slice() {
        let layout = Layout::new::<u32>();
        let mut blob = BlobVec::new(layout);

        let val: u32 = 0;
        unsafe { blob.add(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0]);

        let val: u32 = 32;
        unsafe { blob.add(val) };
        let slice = unsafe { blob.as_slice::<u32>() };

        assert_eq!(slice, &[0, 32]);
    }
}

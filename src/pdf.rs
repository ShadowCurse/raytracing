use rand::Rng;

use crate::hittable::WithHittableTrait;
use crate::onb::Onb;
use crate::vec3::{Point3, Vec3};

pub trait Pdf {
    fn value(&self, direction: &Vec3) -> f32;
    fn generate(&self) -> Vec3;
}

pub struct CosinePdf {
    pub uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: &Vec3) -> Self {
        Self {
            uvw: Onb::new_from_w(w),
        }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f32 {
        let cosine = direction.unit().dot(&self.uvw.w);
        if cosine <= 0.0 {
            0.0
        } else {
            cosine / std::f32::consts::PI
        }
    }

    fn generate(&self) -> Vec3 {
        self.uvw.local_from_vec(&Vec3::random_cosine_direction())
    }
}

pub struct HittablePdf<'a> {
    pub object: &'a WithHittableTrait,
    pub origin: Point3,
}

impl<'a> HittablePdf<'a> {
    pub fn new(object: &'a WithHittableTrait, origin: Point3) -> Self {
        Self { object, origin }
    }
}

impl<'a> Pdf for HittablePdf<'a> {
    fn value(&self, direction: &Vec3) -> f32 {
        self.object.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.object.random(&self.origin)
    }
}

pub struct MixturePdf<'a> {
    pub pdf1: &'a dyn Pdf,
    pub pdf2: &'a dyn Pdf,
}

impl<'a> MixturePdf<'a> {
    pub fn new(pdf1: &'a dyn Pdf, pdf2: &'a dyn Pdf) -> Self {
        Self { pdf1, pdf2 }
    }
}

impl<'a> Pdf for MixturePdf<'a> {
    fn value(&self, direction: &Vec3) -> f32 {
        0.5 * self.pdf1.value(direction) + 0.5 * self.pdf2.value(direction)
    }

    fn generate(&self) -> Vec3 {
        if rand::thread_rng().gen_bool(0.5) {
            self.pdf1.generate()
        } else {
            self.pdf2.generate()
        }
    }
}

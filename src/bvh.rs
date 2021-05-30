use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable, WithHittableTrait};
use crate::ray::Ray;
use crate::world::World;

use std::cmp::Ordering;
use std::sync::Arc;

// TODO refactor
pub struct BVHNode {
    aabb: AABB,
    left: Arc<WithHittableTrait>,
    right: Arc<WithHittableTrait>,
}

impl BVHNode {
    pub fn new(world: &World, time0: f32, time1: f32) -> Self {
        let mut objects = world.objects.clone();
        Self::create_node(&mut objects, 0, time0, time1)
    }
    fn create_node(
        objects: &mut [Arc<WithHittableTrait>],
        mut axis: u8,
        time0: f32,
        time1: f32,
    ) -> Self {
        let cmp = match axis {
            0 => Self::x_cmp,
            1 => Self::y_cmp,
            _ => Self::z_cmp,
        };
        axis = (axis + 1) % 3;
        let span = objects.len();
        let (left, right) = match span {
            1 => (objects[0].clone(), objects[0].clone()),
            2 => match cmp(&objects[0], &objects[1]) {
                Ordering::Less => (objects[0].clone(), objects[1].clone()),
                Ordering::Greater => (objects[1].clone(), objects[0].clone()),
                Ordering::Equal => (objects[0].clone(), objects[1].clone()),
            },
            _ => {
                objects.sort_by(cmp);
                let chunk_size = if span % 2 == 1 {
                    1 + span / 2
                } else {
                    span / 2
                };
                let mut chunks = objects.chunks_mut(chunk_size).collect::<Vec<_>>();
                (
                    Arc::new(Self::create_node(chunks[0], axis, time0, time1))
                        as Arc<WithHittableTrait>,
                    Arc::new(Self::create_node(chunks[1], axis, time0, time1))
                        as Arc<WithHittableTrait>,
                )
            }
        };
        let left_box = left.bounding_box(time0, time1).unwrap();
        let right_box = right.bounding_box(time0, time1).unwrap();
        let aabb = AABB::surrounding_box(left_box, right_box);

        Self { aabb, left, right }
    }
    fn x_cmp(a: &Arc<WithHittableTrait>, b: &Arc<WithHittableTrait>) -> Ordering {
        let box_a = a.bounding_box(0.0, 0.0).unwrap();
        let box_b = b.bounding_box(0.0, 0.0).unwrap();
        box_a.minimum.x.partial_cmp(&box_b.minimum.x).unwrap()
    }
    fn y_cmp(a: &Arc<WithHittableTrait>, b: &Arc<WithHittableTrait>) -> Ordering {
        let box_a = a.bounding_box(0.0, 0.0).unwrap();
        let box_b = b.bounding_box(0.0, 0.0).unwrap();
        box_a.minimum.y.partial_cmp(&box_b.minimum.y).unwrap()
    }
    fn z_cmp(a: &Arc<WithHittableTrait>, b: &Arc<WithHittableTrait>) -> Ordering {
        let box_a = a.bounding_box(0.0, 0.0).unwrap();
        let box_b = b.bounding_box(0.0, 0.0).unwrap();
        box_a.minimum.z.partial_cmp(&box_b.minimum.z).unwrap()
    }
}

impl Hittable for BVHNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        return if self.aabb.hit(ray, t_min, t_max) {
            let l_hit = self.left.hit(ray, t_min, t_max);
            let r_hit = self.right.hit(ray, t_min, t_max);
            match (&l_hit, &r_hit) {
                (Some(lh), Some(rh)) => {
                    if lh.t < rh.t {
                        l_hit
                    } else {
                        r_hit
                    }
                }
                (Some(_), _) => l_hit,
                (_, Some(_)) => r_hit,
                (_, _) => None,
            }
        } else {
            None
        };
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(self.aabb.clone())
    }
}

impl std::fmt::Debug for BVHNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BVHNode")
            .field("aabb", &self.aabb)
            .field("\nleft", &self.left.as_ref().bounding_box(0.0, 0.0))
            .field("\nright", &self.right.as_ref().bounding_box(0.0, 0.0))
            .finish()
    }
}

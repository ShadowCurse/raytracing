use std::cmp::Ordering;
use std::sync::Arc;

use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable, WithHittableTrait};
use crate::ray::Ray;
use crate::world::{World, WorldIndex};
use crate::World3;

enum BVHNIndex {
    Node(usize),
    WorldIndex((WorldIndex, AABB)),
}

struct BVHNode2 {
    left: BVHNIndex,
    right: BVHNIndex,
    aabb: AABB,
}

pub struct BVH {
    world: World3,
    nodes: Vec<BVHNode2>,
}

impl BVH {
    pub fn from_world(world: World3, time0: f32, time1: f32) -> Self {
        let mut volumes = world.volumes();
        let mut nodes = Vec::new();
        let _index = Self::create_node(&mut nodes, &mut volumes, 0, time0, time1);
        Self { world, nodes }
    }

    fn create_node(
        nodes: &mut Vec<BVHNode2>,
        volumes: &mut [(WorldIndex, AABB)],
        axis: u8,
        time0: f32,
        time1: f32,
    ) -> BVHNIndex {
        let cmp = match axis {
            0 => Self::x_cmp,
            1 => Self::y_cmp,
            _ => Self::z_cmp,
        };
        let axis = (axis + 1) % 3;
        let (left, right) = match volumes.len() {
            1 => return BVHNIndex::WorldIndex(volumes[0]),
            2 => match cmp(&volumes[0], &volumes[1]) {
                Ordering::Less => (
                    BVHNIndex::WorldIndex(volumes[0]),
                    BVHNIndex::WorldIndex(volumes[1]),
                ),
                Ordering::Greater => (
                    BVHNIndex::WorldIndex(volumes[1]),
                    BVHNIndex::WorldIndex(volumes[0]),
                ),
                Ordering::Equal => (
                    BVHNIndex::WorldIndex(volumes[0]),
                    BVHNIndex::WorldIndex(volumes[1]),
                ),
            },
            _ => {
                volumes.sort_by(cmp);
                let (left, right) = volumes.split_at_mut(volumes.len() / 2);
                (
                    Self::create_node(nodes, left, axis, time0, time1),
                    Self::create_node(nodes, right, axis, time0, time1),
                )
            }
        };
        let left_box = match left {
            BVHNIndex::Node(index) => nodes[index].aabb,
            BVHNIndex::WorldIndex((_, aabb)) => aabb,
        };
        let right_box = match right {
            BVHNIndex::Node(index) => nodes[index].aabb,
            BVHNIndex::WorldIndex((_, aabb)) => aabb,
        };
        let aabb = AABB::surrounding_box(left_box, right_box);

        nodes.push(BVHNode2 { left, right, aabb });
        BVHNIndex::Node(nodes.len() - 1)
    }
    fn x_cmp(box_a: &(WorldIndex, AABB), box_b: &(WorldIndex, AABB)) -> Ordering {
        box_a.1.minimum.x.partial_cmp(&box_b.1.minimum.x).unwrap()
    }
    fn y_cmp(box_a: &(WorldIndex, AABB), box_b: &(WorldIndex, AABB)) -> Ordering {
        box_a.1.minimum.y.partial_cmp(&box_b.1.minimum.y).unwrap()
    }
    fn z_cmp(box_a: &(WorldIndex, AABB), box_b: &(WorldIndex, AABB)) -> Ordering {
        box_a.1.minimum.z.partial_cmp(&box_b.1.minimum.z).unwrap()
    }

    fn check_node(
        &self,
        index: usize,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
    ) -> Option<(WorldIndex, AABB)> {
        let node = &self.nodes[index];
        if node.aabb.hit(ray, t_min, t_max) {
            let left = match node.left {
                BVHNIndex::Node(index) => self.check_node(index, ray, t_min, t_max),
                BVHNIndex::WorldIndex((world_index, aabb)) => Some((world_index, aabb)),
            };
            let right = match node.right {
                BVHNIndex::Node(index) => self.check_node(index, ray, t_min, t_max),
                BVHNIndex::WorldIndex((world_index, aabb)) => Some((world_index, aabb)),
            };
            match (left, right) {
                (Some((left_index, left_aabb)), Some((right_index, right_aabb))) => {
                    match (
                        left_aabb.hit(ray, t_min, t_max),
                        right_aabb.hit(ray, t_min, t_max),
                    ) {
                        (true, true) => {
                            let left_hit = self.world.hit_object(left_index, ray, t_min, t_max);
                            let right_hit = self.world.hit_object(right_index, ray, t_min, t_max);
                            match (left_hit, right_hit) {
                                (Some(lh), Some(rh)) => {
                                    if lh.t < rh.t {
                                        Some((left_index, left_aabb))
                                    } else {
                                        Some((right_index, right_aabb))
                                    }
                                }
                                (Some(_), _) => Some((left_index, left_aabb)),
                                (_, Some(_)) => Some((right_index, right_aabb)),
                                (_, _) => None,
                            }
                        }
                        (false, true) => Some((left_index, left_aabb)),
                        (true, false) => Some((right_index, right_aabb)),
                        _ => None,
                    }
                }
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Hittable for BVH {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self.check_node(self.nodes.len() - 1, ray, t_min, t_max) {
            Some((index, _)) => self.world.hit_object(index, ray, t_min, t_max),
            None => None,
        }
    }

    fn bounding_box(&self, _: f32, _: f32) -> Option<AABB> {
        Some(self.nodes[self.nodes.len() - 1].aabb)
    }
}

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
        Some(self.aabb)
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

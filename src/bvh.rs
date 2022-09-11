use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::world::{World, WorldIndex};
use std::cmp::Ordering;

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
    world: World,
    nodes: Vec<BVHNode2>,
}

impl BVH {
    pub fn from_world(world: World, time0: f32, time1: f32) -> Self {
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
    ) -> Option<&(WorldIndex, AABB)> {
        let node = &self.nodes[index];
        if node.aabb.hit(ray, t_min, t_max) {
            let left = match node.left {
                BVHNIndex::Node(index) => self.check_node(index, ray, t_min, t_max),
                BVHNIndex::WorldIndex(ref l) => Some(l),
            };
            let right = match node.right {
                BVHNIndex::Node(index) => self.check_node(index, ray, t_min, t_max),
                BVHNIndex::WorldIndex(ref r) => Some(r),
            };
            match (left, right) {
                (Some(l), Some(r)) => {
                    let (left_index, left_aabb) = l;
                    let (right_index, right_aabb) = r;
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
                                        Some(l)
                                    } else {
                                        Some(r)
                                    }
                                }
                                (Some(_), _) => Some(l),
                                (_, Some(_)) => Some(r),
                                (_, _) => None,
                            }
                        }
                        (false, true) => Some(l),
                        (true, false) => Some(r),
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
        let (index, _) = self.check_node(self.nodes.len() - 1, ray, t_min, t_max)?;
        self.world.hit_object(&index, ray, t_min, t_max)
    }

    fn bounding_box(&self) -> AABB {
        self.nodes[self.nodes.len() - 1].aabb
    }
}

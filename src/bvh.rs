use crate::aabb::AABB;
use crate::hittable::{HitRecord, Hittable, WithHittableTrait};
use crate::ray::Ray;
use crate::world::World;

use std::cmp::Ordering;
use std::sync::Arc;

pub struct BVHNode {
    aabb: AABB,
    left: Arc<WithHittableTrait>,
    right: Arc<WithHittableTrait>,
}

impl BVHNode {
    pub fn new(world: &World, time0: f32, time1: f32) -> Self {
        let mut objects = world.objects.clone();
        Self::create_node(&mut objects, time0, time1)
    }
    fn create_node(objects: &mut [Arc<WithHittableTrait>], time0: f32, time1: f32) -> Self {
        // println!("objects size: {}", objects.len());
        use rand::distributions::Distribution;
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(0, 2);
        let axis = uniform.sample(&mut rng);
        let cmp = match axis {
            0 => Self::x_cmp,
            1 => Self::y_cmp,
            _ => Self::z_cmp,
        };
        let span = objects.len();
        let (left, right) = match span {
            1 => (objects[0].clone(), objects[0].clone()),
            2 => match cmp(&objects[0], &objects[1]) {
                Ordering::Less => (objects[0].clone(), objects[1].clone()),
                Ordering::Greater => (objects[1].clone(), objects[0].clone()),
                _ => (objects[0].clone(), objects[1].clone()),
            },
            _ => {
                objects.sort_by(cmp);
                let mut chunks = objects.chunks_mut(1 + span / 2).collect::<Vec<_>>();
                // println!("chunks size: {}", chunks.len());
                (
                    Arc::new(Self::create_node(chunks[0], time0, time1)) as Arc<WithHittableTrait>,
                    Arc::new(Self::create_node(chunks[1], time0, time1)) as Arc<WithHittableTrait>,
                )
            }
        };
        let left_box = left.bounding_box(time0, time1).unwrap();
        let right_box = right.bounding_box(time0, time1).unwrap();
        let aabb = AABB::surrounding_box(left_box, right_box);

        // println!(
        //     "created aabb: {:?} left: {:?}, right: {:?}",
        //     aabb, left_box, right_box
        // );

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

#[cfg(test)]
mod test {
    use crate::material::{Dielectric, Lambertian, Metal};
    use crate::sphere::Sphere;
    use crate::vec3::{Color, Point3};
    use crate::world::World;

    use crate::bvh::BVHNode;
    use std::sync::Arc;

    #[test]
    fn bvh_creation_test() {
        let mut world = World::default();

        let material_ground = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
        world.add_object(Arc::new(Sphere::new(
            Point3::new(0.0, -1000.0, 0.0),
            1000.0,
            material_ground,
        )));

        let material_left = Arc::new(Dielectric::new(1.5));
        world.add_object(Arc::new(Sphere::new(
            Point3::new(0.0, 1.0, 0.0),
            1.0,
            material_left,
        )));
        let material_center = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
        world.add_object(Arc::new(Sphere::new(
            Point3::new(-4.0, 1.0, 0.0),
            1.0,
            material_center,
        )));
        let material_right = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
        world.add_object(Arc::new(Sphere::new(
            Point3::new(4.0, 1.0, 0.0),
            1.0,
            material_right,
        )));
        let material_right = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
        world.add_object(Arc::new(Sphere::new(
            Point3::new(6.0, 1.0, 0.0),
            1.0,
            material_right,
        )));

        let bvh = BVHNode::new(&world, 0.0, 1.0);
        // println!("{:?}", bvh);
    }
}

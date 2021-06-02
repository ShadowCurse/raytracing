mod aabb;
mod bvh;
mod camera;
mod hittable;
mod material;
mod perlin;
mod ray;
mod renderer;
mod sphere;
mod texture;
mod vec3;
mod world;

use camera::*;
use material::*;
use perlin::*;
use renderer::*;
use sphere::*;
use texture::*;
use vec3::*;
use world::*;

use std::sync::Arc;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;
const MAX_DEPTH: u32 = 50;

pub fn main() -> Result<(), String> {
    let world = two_perlin_spheres();

    // let now = std::time::Instant::now();
    // let bvh = BVHNode::new(&world, 0.0, 1.0);
    // let delta = std::time::Instant::now() - now;
    // println!("bvh created in {}ms", delta.as_millis());

    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Point3::new(0.0, 1.0, 0.0);
    let dits_to_focus = 10.0;
    let aperture = 0.0;

    let camera = Camera::new(
        &look_from,
        &look_at,
        &v_up,
        20.0,
        ASPECT_RATIO,
        aperture,
        dits_to_focus,
        0.0,
        1.0,
    );

    let mut renderer = Renderer::new(SCREEN_WIDTH, SCREEN_HEIGHT, SAMPLES_PER_PIXEL, MAX_DEPTH)?;
    renderer.render(&world, &camera)?;
    renderer.present()?;
    Ok(())
}

fn world_1() -> World {
    let mut world = World::default();

    let material_ground = Arc::new(Lambertian::new(Arc::new(CheckerTexture::from_colors(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ))));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_ground,
    )));

    use rand::distributions::Distribution;
    let mut rng = rand::thread_rng();
    let uniform = rand::distributions::Uniform::new(0.0, 1.0);
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = uniform.sample(&mut rng);
            let center = Point3::new(
                a as f32 + 0.9 * uniform.sample(&mut rng),
                0.2,
                b as f32 + 0.9 * uniform.sample(&mut rng),
            );
            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    let material_lambertian = Arc::new(Lambertian::new(Arc::new(
                        SolidTexture::from_color(Color::random(0.0, 0.5)),
                    )));
                    let mut rng = rand::thread_rng();
                    let uniform = rand::distributions::Uniform::new(0.0, 0.5);
                    let center2 = center + Vec3::new(0.0, uniform.sample(&mut rng), 0.0);
                    world.add_object(Arc::new(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        1.0,
                        0.2,
                        material_lambertian,
                    )));
                } else if choose_mat < 0.95 {
                    let material_metal = Arc::new(Metal::new(
                        Color::random(0.5, 1.0),
                        uniform.sample(&mut rng),
                    ));
                    world.add_object(Arc::new(Sphere::new(center, 0.2, material_metal)));
                } else {
                    let material_dielectric = Arc::new(Dielectric::new(1.5));
                    world.add_object(Arc::new(Sphere::new(center, 0.2, material_dielectric)));
                }
            }
        }
    }

    let material_center = Arc::new(Dielectric::new(1.5));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material_center,
    )));
    let material_left = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_color(
        Color::new(0.4, 0.2, 0.1),
    ))));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material_left,
    )));
    let material_right = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material_right,
    )));

    world
}

fn two_shperes() -> World {
    let mut world = World::default();

    let checker = Arc::new(Lambertian::new(Arc::new(CheckerTexture::from_colors(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ))));

    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        checker.clone(),
    )));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        checker.clone(),
    )));

    world
}

fn two_perlin_spheres() -> World {
    let mut world = World::default();

    let checker = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(4.0))));

    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        checker.clone(),
    )));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        checker.clone(),
    )));

    world
}

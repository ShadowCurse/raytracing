mod aabb;
mod bvh;
mod camera;
mod hittable;
mod material;
mod objects;
mod perlin;
mod ray;
mod renderer;
mod texture;
mod vec3;
mod world;

use camera::*;
use material::*;
use objects::*;
use renderer::*;
use texture::*;
use vec3::*;
use world::*;

use std::sync::Arc;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 100;
const MAX_DEPTH: u32 = 50;

pub fn main() -> Result<(), String> {
    let world = cornell_box();

    // let now = std::time::Instant::now();
    // let bvh = BVHNode::new(&world, 0.0, 1.0);
    // let delta = std::time::Instant::now() - now;
    // println!("bvh created in {}ms", delta.as_millis());

    let look_from = Point3::new(278.0, 278.0, -800.0);
    let look_at = Point3::new(278.0, 278.0, 0.0);
    let v_up = Point3::new(0.0, 1.0, 0.0);
    let dits_to_focus = 10.0;
    let aperture = 0.0;

    let camera = Camera::new(
        &look_from,
        &look_at,
        &v_up,
        40.0,
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
                        Arc::new(SolidTexture::from_color(Color::random(0.5, 1.0))),
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
    let material_right = Arc::new(Metal::new(
        Arc::new(SolidTexture::from_color(Color::new(0.7, 0.6, 0.5))),
        0.0,
    ));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material_right,
    )));

    world
}

fn two_spheres() -> World {
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
fn earth() -> World {
    let mut world = World::default();

    let texture = ImageTexture::new("textures/earthmap.jpg").unwrap();
    let material = Arc::new(Lambertian::new(Arc::new(texture)));

    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        material,
    )));

    world
}

fn simple_light() -> World {
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
        checker,
    )));

    let difflight = Arc::new(DiffuseLight::new(Arc::new(SolidTexture::from_rgb(
        4.0, 4.0, 4.0,
    ))));
    world.add_object(Arc::new(XYRect::new(3.0, 5.0, 1.0, 3.0, -2.0, difflight)));

    world
}

fn cornell_box() -> World {
    let mut world = World::default();

    let red = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.65, 0.05, 0.05,
    ))));
    let white = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.73, 0.73, 0.73,
    ))));
    let green = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.12, 0.45, 0.15,
    ))));
    let light = Arc::new(DiffuseLight::new(Arc::new(SolidTexture::from_rgb(
        150.0, 150.0, 150.0,
    ))));

    world.add_object(Arc::new(YZRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        green.clone(),
    )));
    world.add_object(Arc::new(YZRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        red.clone(),
    )));
    world.add_object(Arc::new(XZRect::new(
        213.0,
        343.0,
        227.0,
        332.0,
        554.0,
        light.clone(),
    )));
    world.add_object(Arc::new(XZRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        0.0,
        white.clone(),
    )));
    world.add_object(Arc::new(XZRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    )));
    world.add_object(Arc::new(XYRect::new(
        0.0,
        555.0,
        0.0,
        555.0,
        555.0,
        white.clone(),
    )));
    world.add_object(Arc::new( Box3d::new(
        Point3::new(130.0, 0.0, 65.0),
        Point3::new(295.0, 165.0, 230.0),
        white.clone(),
    )));
    world.add_object(Arc::new(Box3d::new(
        Point3::new(265.0, 0.0, 295.0),
        Point3::new(430.0, 330.0, 460.0),
        white.clone(),
    )));


    world
}

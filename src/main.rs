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

use crate::bvh::BVHNode;
use crate::hittable::{ConstantMedium, Rotate, Translate};
use rand::Rng;
use std::sync::Arc;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 50;
const MAX_DEPTH: u32 = 50;

pub fn main() -> Result<(), String> {
    let world = final_scene();

    // let now = std::time::Instant::now();
    // let bvh = BVHNode::new(&world, 0.0, 1.0);
    // let delta = std::time::Instant::now() - now;
    // println!("bvh created in {}ms", delta.as_millis());

    let look_from = Point3::new(478.0, 278.0, -600.0);
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
    world.add_object(Arc::new(Translate::new(
        Arc::new(Rotate::new(
            Arc::new(Box3d::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(165.0, 330.0, 165.0),
                white.clone(),
            )),
            15.0,
        )),
        Vec3::new(265.0, 0.0, 295.0),
    )));

    world.add_object(Arc::new(Translate::new(
        Arc::new(Rotate::new(
            Arc::new(Box3d::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(165.0, 165.0, 165.0),
                white.clone(),
            )),
            -18.0,
        )),
        Vec3::new(130.0, 0.0, 65.0),
    )));

    world
}

fn cornell_smoke() -> World {
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
        7.0, 7.0, 7.0,
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
        113.0,
        443.0,
        127.0,
        432.0,
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
    world.add_object(Arc::new(ConstantMedium::new(
        Arc::new(Translate::new(
            Arc::new(Rotate::new(
                Arc::new(Box3d::new(
                    Point3::new(0.0, 0.0, 0.0),
                    Point3::new(165.0, 330.0, 165.0),
                    white.clone(),
                )),
                15.0,
            )),
            Vec3::new(265.0, 0.0, 295.0),
        )),
        0.01,
        Arc::new(Isotropic::new(Arc::new(SolidTexture::from_color(
            Color::new(0.0, 0.0, 0.0),
        )))),
    )));

    world.add_object(Arc::new(ConstantMedium::new(
        Arc::new(Translate::new(
            Arc::new(Rotate::new(
                Arc::new(Box3d::new(
                    Point3::new(0.0, 0.0, 0.0),
                    Point3::new(165.0, 165.0, 165.0),
                    white.clone(),
                )),
                -18.0,
            )),
            Vec3::new(130.0, 0.0, 65.0),
        )),
        0.01,
        Arc::new(Isotropic::new(Arc::new(SolidTexture::from_color(
            Color::new(1.0, 1.0, 1.0),
        )))),
    )));

    world
}

fn final_scene() -> World {
    let mut boxes = World::default();

    let ground = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.48, 0.83, 0.53,
    ))));

    const BOXES_PER_SIDE: u32 = 20;
    for i in 0..BOXES_PER_SIDE {
        for j in 0..BOXES_PER_SIDE {
            let w = 100.0;
            let x0 = -1000.0 + i as f32 * w;
            let z0 = -1000.0 + j as f32 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = rand::thread_rng().gen_range(1..101) as f32;
            let z1 = z0 + w;

            boxes.add_object(Arc::new(Box3d::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    let mut world = World::default();

    world.add_object(Arc::new(BVHNode::new(&boxes, 0.0, 1.0)));

    let light = Arc::new(DiffuseLight::new(Arc::new(SolidTexture::from_rgb(
        7.0, 7.0, 7.0,
    ))));

    world.add_object(Arc::new(XZRect::new(
        123.0, 423.0, 147.0, 412.0, 554.0, light,
    )));

    let center1 = Point3::new(400.0, 400.0, 400.0);
    let center2 = center1 + Point3::new(30.0, 0.0, 0.0);
    let moving_sphere_material = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.7, 0.3, 0.1,
    ))));
    world.add_object(Arc::new(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        50.0,
        moving_sphere_material,
    )));

    world.add_object(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(Dielectric::new(1.5)),
    )));

    world.add_object(Arc::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Arc::new(Metal::new(
            Arc::new(SolidTexture::from_rgb(0.8, 0.8, 0.8)),
            1.0,
        )),
    )));

    let boundary = Arc::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::new(Dielectric::new(1.5)),
    ));

    world.add_object(boundary.clone());
    world.add_object(Arc::new(ConstantMedium::new(
        boundary.clone(),
        0.2,
        Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
            0.2, 0.4, 0.9,
        )))),
    )));

    let boundary = Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add_object(Arc::new(ConstantMedium::new(
        boundary.clone(),
        0.0001,
        Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
            1.0, 1.0, 1.0,
        )))),
    )));

    let earth = ImageTexture::new("textures/earthmap.jpg").unwrap();
    let earth_material = Arc::new(Lambertian::new(Arc::new(earth)));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        earth_material,
    )));

    let pertext = Arc::new(Lambertian::new(Arc::new(NoiseTexture::new(0.1))));
    world.add_object(Arc::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        pertext,
    )));

    let mut boxes2 = World::default();
    let white = Arc::new(Lambertian::new(Arc::new(SolidTexture::from_rgb(
        0.73, 0.73, 0.73,
    ))));
    for _ in 0..1000 {
        boxes2.add_object(Arc::new(Sphere::new(
            Point3::random(0.0, 165.0),
            10.0,
            white.clone(),
        )))
    }
    world.add_object(Arc::new(Translate::new(
        Arc::new(Rotate::new(Arc::new(BVHNode::new(&boxes2, 0.0, 1.0)), 15.0)),
        Vec3::new(-100.0, 270.0, 395.0),
    )));

    world
}

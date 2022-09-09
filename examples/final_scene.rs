use rand::Rng;
use rust_raytracing::*;

const ASPECT_RATIO: f32 = 1.0;
const SCREEN_WIDTH: u32 = 600;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 5;
const MAX_DEPTH: u32 = 5;

pub fn main() -> Result<(), String> {
    let world = final_scene();

    let dummy_material = Lambertian::new(SolidTexture::from_color(
        Color::new(0.4, 0.2, 0.1),
    ));
    let light = XZRect::new(123.0, 423.0, 147.0, 412.0, 554.0, dummy_material);

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

    let mut renderer = Renderer::new(
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        SAMPLES_PER_PIXEL,
        MAX_DEPTH,
        Color::new(0.0, 0.0, 0.0),
    )?;
    renderer.render(&world, &camera, Some(&light))?;
    renderer.present()?;
    Ok(())
}

fn final_scene() -> World {
    let mut boxes = World::default();

    let ground = Lambertian::new(SolidTexture::from_rgb(
        0.48, 0.83, 0.53,
    ));

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

            boxes.add(Box3d::new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground,
            ));
        }
    }

    let mut world = World::default();

    world.add(boxes);

    let light = DiffuseLight::new(SolidTexture::from_rgb(
        7.0, 7.0, 7.0,
    ));

    world.add(FlipFace::new(XZRect::new(
        123.0, 423.0, 147.0, 412.0, 554.0, light,
    )));

    let center1 = Point3::new(400.0, 400.0, 400.0);
    let center2 = center1 + Point3::new(30.0, 0.0, 0.0);
    let moving_sphere_material = Lambertian::new(SolidTexture::from_rgb(
        0.7, 0.3, 0.1,
    ));
    world.add(MovingSphere::new(
        center1,
        center2,
        0.0,
        1.0,
        50.0,
        moving_sphere_material,
    ));

    world.add(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Dielectric::new(1.5)),
    );

    world.add(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Metal::new(
            SolidTexture::from_rgb(0.8, 0.8, 0.8),
            1.0,
        ),
    ));

    let boundary = Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Dielectric::new(1.5),
    );

    // world.add(boundary.clone());
    world.add(ConstantMedium::new(
        boundary,
        0.2,
        Lambertian::new(SolidTexture::from_rgb(
            0.2, 0.4, 0.9,
        )),
    ));

    let boundary = Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Dielectric::new(1.5));
    world.add(ConstantMedium::new(
        boundary,
        0.0001,
        Lambertian::new(SolidTexture::from_rgb(
            1.0, 1.0, 1.0,
        )),
    ));

    let earth = ImageTexture::new("textures/earthmap.jpg").unwrap();
    let earth_material = Lambertian::new(earth);
    world.add(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        earth_material,
    ));

    let pertext = Lambertian::new(NoiseTexture::new(0.1));
    world.add(Sphere::new(Point3::new(220.0, 280.0, 300.0), 80.0, pertext));

    // TODO rethink Translation and Rotation
    let mut boxes2 = World::default();
    let white = Lambertian::new(SolidTexture::from_rgb(
        0.73, 0.73, 0.73,
    ));
    for _ in 0..1000 {
        boxes2.add(Sphere::new(
            Point3::random(0.0, 165.0),
            10.0,
            white,
        ));
    }
    world.add(Translate::new(
        Rotate::new(boxes2, 15.0),
        Vec3::new(-100.0, 270.0, 395.0),
    ));

    world
}

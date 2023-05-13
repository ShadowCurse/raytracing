use rust_raytracing::*;

const ASPECT_RATIO: f32 = 1.0;
const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 5;
const MAX_DEPTH: u32 = 5;

pub fn main() -> Result<(), String> {
    let world = cornell_box();

    let mut lights = World::default();
    let dummy_material = Lambertian::new(SolidTexture::from_color(Color::new(0.4, 0.2, 0.1)));
    lights.add(XZRect::new(
        213.0,
        343.0,
        227.0,
        332.0,
        554.0,
        dummy_material,
    ));
    lights.add(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        dummy_material,
    ));

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
    renderer.render(&world, &camera, Some(&lights))?;
    renderer.present()?;
    Ok(())
}

fn cornell_box() -> World {
    let mut world = World::default();

    let red = Lambertian::new(SolidTexture::from_rgb(0.65, 0.05, 0.05));
    let white = Lambertian::new(SolidTexture::from_rgb(0.73, 0.73, 0.73));
    let green = Lambertian::new(SolidTexture::from_rgb(0.12, 0.45, 0.15));
    let light = DiffuseLight::new(SolidTexture::from_rgb(15.0, 15.0, 15.0));

    world.add(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green));
    world.add(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red));
    world.add(FlipFace::new(XZRect::new(
        213.0, 343.0, 227.0, 332.0, 554.0, light,
    )));
    world.add(XZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, white));
    world.add(XZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white));
    world.add(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white));
    world.add(Translate::new(
        Rotate::new(
            Box3d::new(
                Point3::new(0.0, 0.0, 0.0),
                Point3::new(165.0, 330.0, 165.0),
                white,
            ),
            15.0,
        ),
        Vec3::new(265.0, 0.0, 295.0),
    ));

    // world.add(Translate::new(
    //     Rotate::new(
    //         Box3d::new(
    //             Point3::new(0.0, 0.0, 0.0),
    //             Point3::new(165.0, 165.0, 165.0),
    //             white,
    //         ),
    //         -18.0,
    //     ),
    //     Vec3::new(130.0, 0.0, 65.0),
    // ));

    world.add(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        Dielectric::new(2.0),
    ));

    world
}

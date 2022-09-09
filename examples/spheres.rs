use rust_raytracing::*;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 600;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const SAMPLES_PER_PIXEL: u32 = 10;
const MAX_DEPTH: u32 = 10;

pub fn main() -> Result<(), String> {
    let world = scene();

    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Point3::new(0.0, 1.0, 0.0);
    let dits_to_focus = 10.0;
    let aperture = 0.1;

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
        Color::new(0.8, 0.8, 0.8),
    )?;
    renderer.render::<_, World>(&world, &camera, None)?;
    renderer.present()?;
    Ok(())
}

fn scene() -> World {
    let mut world = World::default();

    let material_ground = Lambertian::new(CheckerTexture::from_colors(
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    world.add(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_ground,
    ));

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
                    let material_lambertian =
                        Lambertian::new(SolidTexture::from_color(Color::random(0.0, 0.5)));
                    let mut rng = rand::thread_rng();
                    let uniform = rand::distributions::Uniform::new(0.0, 0.5);
                    let center2 = center + Vec3::new(0.0, uniform.sample(&mut rng), 0.0);
                    world.add(MovingSphere::new(
                        center,
                        center2,
                        0.0,
                        1.0,
                        0.2,
                        material_lambertian,
                    ));
                } else if choose_mat < 0.95 {
                    let material_metal = Metal::new(
                        SolidTexture::from_color(Color::random(0.5, 1.0)),
                        uniform.sample(&mut rng),
                    );
                    world.add(Sphere::new(center, 0.2, material_metal));
                } else {
                    let material_dielectric = Dielectric::new(1.5);
                    world.add(Sphere::new(center, 0.2, material_dielectric));
                }
            }
        }
    }

    let material_center = Dielectric::new(1.5);
    world.add(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material_center,
    ));
    let material_left = Lambertian::new(SolidTexture::from_color(Color::new(0.4, 0.2, 0.1)));
    world.add(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, material_left));
    let material_right = Metal::new(SolidTexture::from_color(Color::new(0.7, 0.6, 0.5)), 0.0);
    world.add(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, material_right));

    world
}

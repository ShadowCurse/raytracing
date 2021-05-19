use rand::distributions::Distribution;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::rc::Rc;

mod camera;
mod hittable;
mod material;
mod ray;
mod sphere;
mod vec3;
mod world;

use camera::*;
use hittable::*;
use material::*;
use ray::*;
use sphere::*;
use vec3::*;
use world::*;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const BUFFER_LENGTH: u32 = SCREEN_WIDTH * SCREEN_HEIGHT * 3;
const PITCH: u32 = SCREEN_WIDTH * 3;
const SAMPLES_PER_PIXEL: u32 = 100;
const MAX_DEPTH: u32 = 50;

fn ray_color(ray: &Ray, world: &World, depth: u32) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }
    if let Some(hit) = world.hit(&ray, 0.001, f32::INFINITY) {
        return if let Some((scatter_ray, scatter_color)) = hit.scatter(ray) {
            scatter_color * ray_color(&scatter_ray, world, depth - 1)
        } else {
            Color::new(0.0, 0.0, 0.0)
        };
    }
    let unit_direction = ray.direction.unit();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn write_pixel(buffer: &mut [u8], x: u32, y: u32, color: &Color, samples_per_pixel: u32) {
    let r = (color.x / samples_per_pixel as f32).sqrt();
    let g = (color.y / samples_per_pixel as f32).sqrt();
    let b = (color.z / samples_per_pixel as f32).sqrt();

    let offset = (SCREEN_HEIGHT - 1 - y) as usize * PITCH as usize + x as usize * 3; // for OpenGl reverse y coord
    buffer[offset] = (255.999 * r.clamp(0.0, 0.999)) as u8;
    buffer[offset + 1] = (255.999 * g.clamp(0.0, 0.999)) as u8;
    buffer[offset + 2] = (255.999 * b.clamp(0.0, 0.999)) as u8;
}

fn create_texture() -> Vec<u8> {
    let mut world = World::default();

    let material_ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    world.add_object(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material_ground,
    )));

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
                    let material_lambertian = Rc::new(Lambertian::new(Color::random(0.0, 0.1)));
                    world.add_object(Box::new(Sphere::new(center, 0.2, material_lambertian)));
                } else if choose_mat < 0.95 {
                    let material_metal = Rc::new(Metal::new(
                        Color::random(0.5, 1.0),
                        uniform.sample(&mut rng),
                    ));
                    world.add_object(Box::new(Sphere::new(center, 0.2, material_metal)));
                } else {
                    let material_dielectric = Rc::new(Dielectric::new(1.5));
                    world.add_object(Box::new(Sphere::new(center, 0.2, material_dielectric)));
                }
            }
        }
    }

    let material_left = Rc::new(Dielectric::new(1.5));
    world.add_object(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material_left,
    )));
    let material_center = Rc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add_object(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material_center,
    )));
    let material_right = Rc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add_object(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material_right,
    )));

    let look_from = Point3::new(13.0, 2.0, 3.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let v_up = Point3::new(0.0, 1.0, 0.0);
    let dits_to_focus = 10.0; //(look_from - look_at).length();
    let aperture = 0.1;

    let camera = Camera::new(
        &look_from,
        &look_at,
        &v_up,
        20.0,
        ASPECT_RATIO,
        aperture,
        dits_to_focus,
    );

    let mut buffer = vec![0u8; BUFFER_LENGTH as usize];
    let mut rng = rand::thread_rng();
    let uniform = rand::distributions::Uniform::new(0.0, 1.0);
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f32 + uniform.sample(&mut rng)) / (SCREEN_WIDTH - 1) as f32;
                let v = (y as f32 + uniform.sample(&mut rng)) / (SCREEN_HEIGHT - 1) as f32;
                let r = camera.get_ray(u, v);
                color += ray_color(&r, &world, MAX_DEPTH);
            }
            write_pixel(&mut buffer, x, y, &color, SAMPLES_PER_PIXEL);
        }
        println!("Progress: {}%", y as f32 * 100.0 / SCREEN_HEIGHT as f32);
    }
    buffer
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust_raytracing", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
        .map_err(|e| e.to_string())?;

    let now = std::time::Instant::now();
    let buffer = create_texture();
    let delta = std::time::Instant::now() - now;
    println!("Texture created in {}ms", delta.as_millis());

    texture.update(None, &buffer, PITCH as usize).unwrap();

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
    }

    Ok(())
}

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

mod camera;
mod hittable;
mod ray;
mod sphere;
mod vec3;
mod world;

use crate::hittable::Hittable;
use camera::*;
use ray::*;
use sphere::*;
use vec3::*;
use world::*;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const BUFFER_LENGTH: u32 = SCREEN_WIDTH * SCREEN_HEIGHT * 3;
const PITCH: u32 = SCREEN_WIDTH * 3;
const SAMPLES_PER_PIXEL: u32 = 10;

fn ray_color(ray: &Ray, world: &World) -> Color {
    if let Some(record) = world.hit(&ray, 0.0, f32::INFINITY) {
        return 0.5 * (record.normal + Color::new(1.0, 1.0, 1.0));
    }
    let unit_direction = ray.direction.unit();
    let t = 0.5 * (unit_direction.y + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn write_pixel(buffer: &mut [u8], x: u32, y: u32, color: &Color, samples_per_pixel: u32) {
    let r = color.x / samples_per_pixel as f32;
    let g = color.y / samples_per_pixel as f32;
    let b = color.z / samples_per_pixel as f32;

    let offset = (SCREEN_HEIGHT - 1 - y) as usize * PITCH as usize + x as usize * 3; // for OpenGl reverse y coord
    buffer[offset] = (255.999 * r.clamp(0.0, 0.999)) as u8;
    buffer[offset + 1] = (255.999 * g.clamp(0.0, 0.999)) as u8;
    buffer[offset + 2] = (255.999 * b.clamp(0.0, 0.999)) as u8;
}

fn create_texture() -> Vec<u8> {
    let mut world = World::default();
    world.add_object(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));
    world.add_object(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));

    let camera = Camera::new(ASPECT_RATIO);

    let mut buffer = vec![0u8; BUFFER_LENGTH as usize];
    let mut rng = rand::thread_rng();
    let mut uniform = rand::distributions::Uniform::new(0.0, 1.0);
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let mut color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f32 + uniform.sample(&mut rng)) / (SCREEN_WIDTH - 1) as f32;
                let v = (y as f32 + uniform.sample(&mut rng)) / (SCREEN_HEIGHT - 1) as f32;
                let r = camera.get_ray(u, v);
                color += ray_color(&r, &world);
            }
            write_pixel(&mut buffer, x, y, &color, SAMPLES_PER_PIXEL);
        }
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

    texture.update(None, &buffer, PITCH as usize);

    canvas.clear();
    canvas.copy(&texture, None, None)?;
    canvas.present();

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
    }

    Ok(())
}

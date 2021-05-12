use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

mod ray;
mod vec3;

use ray::*;
use vec3::*;

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const SCREEN_WIDTH: u32 = 400;
const SCREEN_HEIGHT: u32 = (SCREEN_WIDTH as f32 / ASPECT_RATIO) as u32;
const BUFFER_LENGTH: u32 = SCREEN_WIDTH * SCREEN_HEIGHT * 3;
const PITCH: u32 = SCREEN_WIDTH * 3;

fn ray_color(ray: &Ray) -> Color {
    let unit_direction = ray.direction.unit();
    let t = 0.5 * (-unit_direction.y + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - t) + Color::new(0.5, 0.7, 1.0) * t
}

fn create_texture() -> Vec<u8> {
    let viewport_height = 2.0;
    let viewport_width = ASPECT_RATIO * viewport_height;
    let focal_length = 1.0;

    let origin = Point3::new(0.0, 0.0, 0.0);
    let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
    let vertical = Vec3::new(0.0, viewport_height, 0.0);
    let lower_left_corner =
        origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

    let mut buffer = vec![0u8; BUFFER_LENGTH as usize];
    for y in 0..SCREEN_HEIGHT {
        for x in 0..SCREEN_WIDTH {
            let u = x as f32 / (SCREEN_WIDTH - 1) as f32;
            let v = y as f32 / (SCREEN_HEIGHT - 1) as f32;

            let dir = lower_left_corner + horizontal * u + vertical * v - origin;
            let r = Ray::new(origin, dir);
            let color = ray_color(&r);

            let offset = y as usize * PITCH as usize + x as usize * 3;
            buffer[offset] = (255.999 * color.x) as u8;
            buffer[offset + 1] = (255.999 * color.y) as u8;
            buffer[offset + 2] = (255.999 * color.z) as u8;
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

    let buffer = create_texture();
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

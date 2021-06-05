use rand::distributions::Distribution;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use crate::camera::*;
use crate::hittable::WithHittableTrait;
use crate::ray::*;
use crate::vec3::*;

pub struct Renderer {
    screen_width: u32,
    screen_height: u32,
    samples_per_pixel: u32,
    max_depth: u32,
    pitch: u32,
    buffer: Vec<u8>,
}

impl<'a> Renderer {
    pub fn new(
        screen_width: u32,
        screen_height: u32,
        samples_per_pixel: u32,
        max_depth: u32,
    ) -> Result<Self, String> {
        Ok(Self {
            screen_width,
            screen_height,
            samples_per_pixel,
            max_depth,
            pitch: screen_width * 3,
            buffer: vec![0u8; (screen_width * screen_height * 3) as usize],
        })
    }

    pub fn render(&mut self, hittable: &WithHittableTrait, camera: &Camera) -> Result<(), String> {
        let thread_num = 16;
        let tile_height = self.screen_height / thread_num;

        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let samples_per_pixel = self.samples_per_pixel;
        let max_depth = self.max_depth;

        let now = std::time::Instant::now();

        crossbeam::scope(|spawner| {
            self.buffer
                .chunks_mut((tile_height * screen_width * 3) as usize)
                .enumerate()
                .map(|(i, buff)| {
                    let bottom = ((thread_num - 1) - i as u32) * tile_height;
                    let top = bottom + tile_height;
                    spawner.spawn(move |_| {
                        Self::render_tile(
                            buff,
                            (0, top),
                            (screen_width, bottom),
                            (screen_width, screen_height),
                            hittable,
                            camera,
                            samples_per_pixel,
                            max_depth,
                        );
                    })
                })
                .for_each(|_| {});
        })
        .map_err(|e| format!("crossbeam error: {:?}", e))?;

        let delta = std::time::Instant::now() - now;
        println!("Texture created in {}ms", delta.as_millis());
        Ok(())
    }

    pub fn present(&mut self) -> Result<(), String> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;

        let window = video_subsystem
            .window("rust_raytracing", self.screen_width, self.screen_height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                self.screen_width,
                self.screen_height,
            )
            .map_err(|e| e.to_string())?;

        texture
            .update(None, &self.buffer, self.pitch as usize)
            .unwrap();

        let mut event_pump = context.event_pump()?;
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

    fn ray_color(
        ray: &Ray,
        hittable: &WithHittableTrait,
        background: &Color,
        max_depth: u32,
    ) -> Color {
        let mut final_color = Color::new(1.0, 1.0, 1.0);
        let mut current_ray = *ray;
        let mut curr_depth: u32 = 0;
        loop {
            curr_depth += 1;
            if curr_depth == max_depth {
                return Color::new(0.0, 0.0, 0.0);
            };
            if let Some(hit) = hittable.hit(&current_ray, 0.001, f32::INFINITY) {
                let emitted = hit.material.unwrap().emit(hit.u, hit.u, &hit.point);
                if let Some((scatter_ray, scatter_color)) = hit.scatter(&current_ray) {
                    current_ray = scatter_ray;
                    final_color = emitted + final_color * scatter_color;
                } else {
                    return emitted;
                }
            } else {
                final_color *= background;
                break;
            }
        }
        final_color
    }

    fn render_tile(
        buffer: &mut [u8],
        top_left: (u32, u32),
        bot_right: (u32, u32),
        window_size: (u32, u32),
        hittable: &'a WithHittableTrait,
        camera: &Camera,
        samples_per_pixel: u32,
        max_depth: u32,
    ) {
        println!(
            "rendering buffer x_range: {:?}, y_range: {:?}",
            (top_left.0..bot_right.0),
            (bot_right.1..top_left.1)
        );
        let mut rng = rand::thread_rng();
        let uniform = rand::distributions::Uniform::new(0.0, 1.0);
        (bot_right.1..top_left.1).for_each(move |y| {
            (top_left.0..bot_right.0).for_each(|x| {
                let mut color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..samples_per_pixel {
                    let u = (x as f32 + uniform.sample(&mut rng)) / (window_size.0 - 1) as f32;
                    let v = (y as f32 + uniform.sample(&mut rng)) / (window_size.1 - 1) as f32;
                    let r = camera.get_ray(u, v);
                    color += Self::ray_color(&r, hittable, &Color::new(0.01, 0.01, 0.01), max_depth);
                }
                Self::write_pixel(buffer, top_left, bot_right, x, y, &color, samples_per_pixel);
            });
        });
    }

    #[inline]
    fn write_pixel(
        buffer: &mut [u8],
        top_left: (u32, u32),
        bot_right: (u32, u32),
        x: u32,
        y: u32,
        color: &Color,
        samples_per_pixel: u32,
    ) {
        let r = (255.999
            * (color.x / samples_per_pixel as f32)
                .sqrt()
                .clamp(0.0, 0.999)) as u8;
        let g = (255.999
            * (color.y / samples_per_pixel as f32)
                .sqrt()
                .clamp(0.0, 0.999)) as u8;
        let b = (255.999
            * (color.z / samples_per_pixel as f32)
                .sqrt()
                .clamp(0.0, 0.999)) as u8;

        let pitch = (bot_right.0 - top_left.0) * 3;
        let offset = (top_left.1 - 1 - y) as usize * pitch as usize + x as usize * 3; // for OpenGl reverse y coord
        buffer[offset] = r;
        buffer[offset + 1] = g;
        buffer[offset + 2] = b;
    }
}

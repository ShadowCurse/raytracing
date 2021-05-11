use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

const IMAGE_WIDTH: u32 = 256;
const IMAGE_HEIGHT: u32 = 256;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust_raytracing", IMAGE_WIDTH, IMAGE_WIDTH)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, IMAGE_WIDTH, IMAGE_HEIGHT)
        .map_err(|e| e.to_string())?;

    let pitch = (IMAGE_WIDTH * 3) as usize;
    let mut buffer = vec![0u8; (IMAGE_WIDTH * IMAGE_HEIGHT * 3) as usize];
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let r = x as f32 / (IMAGE_WIDTH - 1) as f32;
            let g = y as f32 / (IMAGE_HEIGHT - 1) as f32;
            let b: f32 = 0.25;

            let ir = 255.999 * r;
            let ig = 255.999 * g;
            let ib = 255.999 * b;

            let offset = y as usize * pitch + x as usize * 3;
            buffer[offset] = ir as u8;
            buffer[offset + 1] = ig as u8;
            buffer[offset + 2] = ib as u8;
        }
    }
    texture.update(None, &buffer, pitch);

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

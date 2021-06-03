use crate::perlin::Perlin;
use crate::vec3::{Color, Point3};
use image::GenericImageView;
use std::sync::Arc;

pub type WithTexture = dyn Texture + Send + Sync;

pub trait Texture {
    fn color(&self, u: f32, v: f32, point: &Point3) -> Color;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct SolidTexture {
    pub color: Color,
}

impl SolidTexture {
    pub fn from_color(color: Color) -> Self {
        Self { color }
    }
    pub fn from_rgb(red: f32, green: f32, blue: f32) -> Self {
        Self {
            color: Color::new(red, green, blue),
        }
    }
}

impl Texture for SolidTexture {
    fn color(&self, _: f32, _: f32, _: &Point3) -> Color {
        self.color
    }
}

pub struct CheckerTexture {
    odd: Arc<WithTexture>,
    even: Arc<WithTexture>,
}

impl CheckerTexture {
    pub fn from_textures(odd: Arc<WithTexture>, even: Arc<WithTexture>) -> Self {
        Self { odd, even }
    }
    pub fn from_colors(odd: Color, even: Color) -> Self {
        Self {
            odd: Arc::new(SolidTexture::from_color(odd)),
            even: Arc::new(SolidTexture::from_color(even)),
        }
    }
}

impl Texture for CheckerTexture {
    fn color(&self, u: f32, v: f32, point: &Point3) -> Color {
        let sines = (10.0 * point.x).sin() * (10.0 * point.y).sin() * (10.0 * point.z).sin();
        return if sines < 0.0 {
            self.odd.color(u, v, point)
        } else {
            self.even.color(u, v, point)
        };
    }
}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f32,
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn color(&self, _: f32, _: f32, point: &Point3) -> Color {
        Color::new(1.0, 1.0, 1.0)
            * 0.5
            * (1.0 + (self.scale * point.z + 10.0 * self.noise.turb(point, 2)).sin())
    }
}

pub struct ImageTexture {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    pub fn new(path: &str) -> Result<Self, image::error::ImageError> {
        let img = image::open(path)?;
        Ok(Self {
            data: Vec::from(img.as_bytes()),
            width: img.width(),
            height: img.height(),
        })
    }
}

impl Texture for ImageTexture {
    fn color(&self, u: f32, v: f32, point: &Point3) -> Color {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0);
        let mut i = (u * self.width as f32) as u32;
        let mut j = (v * self.height as f32) as u32;
        if i >= self.width {
            i = self.width - 1;
        }
        if j >= self.height {
            j = self.height - 1;
        }

        let color_scale = 1.0 / 255.0;
        let pixel = j * self.width * 3 + i * 3;
        Color::new(
            color_scale * self.data[pixel as usize] as f32,
            color_scale * self.data[pixel as usize + 1] as f32,
            color_scale * self.data[pixel as usize + 2] as f32,
        )
    }
}

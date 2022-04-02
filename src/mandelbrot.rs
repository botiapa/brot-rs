use std::ops::Range;

use angular_units::Turns;
use image::RgbImage;
use num_complex::Complex;
use prisma::{FromColor, Hsv, Rgb};
use rayon::prelude::*;

const MAX_ITER: f32 = 150.0;

const PIXEL_CHUNK: u32 = 10000;

pub const RE_START: f32 = -2.0; // DEFAULT: -2.0
pub const RE_END: f32 = 0.5; // DEFAULT: 1.0
pub const IM_START: f32 = -1.0; // DEFAULT: -1.0
pub const IM_END: f32 = 1.0; // DEFAULT: 1.0

#[derive(Clone)]
pub struct FractalProperties {
    pub re_start: f32,
    pub re_end: f32,
    pub im_start: f32,
    pub im_end: f32,
    pub scale: f32,
}

impl Default for FractalProperties {
    fn default() -> Self {
        Self {
            re_start: RE_START,
            re_end: RE_END,
            im_start: IM_START,
            im_end: IM_END,
            scale: 1.0,
        }
    }
}

fn mandelbrot(c: Complex<f32>) -> f32 {
    let mut z = Complex::new(0f32, 0f32);
    let mut n = 0f32;
    while z.norm() <= 2.0 && n < MAX_ITER {
        z = z * z + c;
        n += 1.0;
    }
    if n != MAX_ITER {
        return n as f32 + 1.0 - z.norm().log2().log10();
    }
    n
}

fn map_to_screen_space(x: f32, max_x: f32, min_y: f32, max_y: f32) -> f32 {
    min_y + (x / max_x) * (max_y - min_y)
}

pub fn generate_image(width: u32, height: u32, fp: FractalProperties) -> Vec<[u8; 3]> {
    let mut img = RgbImage::new(width, height);

    let total_pixels = width * height;

    let regions: Vec<[u8; 3]> = (0..total_pixels)
        .into_par_iter()
        .step_by(PIXEL_CHUNK as usize)
        .map(|start| {
            let end = total_pixels.min(start + PIXEL_CHUNK);
            calculate_region(start..end, img.width(), img.height(), &fp)
        })
        .flatten()
        .collect();
    regions
}

fn calculate_region(
    mut pixel_range: Range<u32>,
    max_x: u32,
    max_y: u32,
    fp: &FractalProperties,
) -> Vec<[u8; 3]> {
    let mut pixels: Vec<[u8; 3]> = Vec::new();
    for i in &mut pixel_range {
        let x = i % max_x;
        let y = i / max_x;
        pixels.push(calculate_pixel(x, y, max_x, max_y, fp));
    }
    pixels
}

fn calculate_pixel(x: u32, y: u32, max_x: u32, max_y: u32, fp: &FractalProperties) -> [u8; 3] {
    let c = Complex::<f32>::new(
        map_to_screen_space(x as f32, max_x as f32, fp.re_start, fp.re_end) * fp.scale,
        map_to_screen_space(y as f32, max_y as f32, fp.im_start, fp.im_end) * fp.scale,
    );
    let n = mandelbrot(c);

    let hue = Turns((n as f32 / MAX_ITER as f32).min(0.9999999));

    let saturation = 1.0;
    let value = if n < MAX_ITER { 1.0f32 } else { 0.0f32 };
    let hsv = Hsv::new(hue, saturation, value);
    let color = Rgb::from_color(&hsv);
    [
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8,
    ]
}

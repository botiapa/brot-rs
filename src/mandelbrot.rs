use std::{f64::consts::PI, ops::Range};

use angular_units::{Deg, Turns};
use num_complex::Complex;
use prisma::{FromColor, Hsv, Rgb};
use rayon::prelude::*;
use rust_decimal::Decimal;

pub type Float = f64;
const DEFAULT_MAX_ITER: Float = 180.0;

const PIXEL_CHUNK: u32 = 10000;

#[derive(Clone)]
pub struct FractalProperties {
    pub center_x: Float,
    pub center_y: Float,
    pub zoom: Float,
    pub max_iter: Float,
    pub ss_factor: usize,
}

impl Default for FractalProperties {
    fn default() -> Self {
        Self {
            center_x: 0 as Float,
            center_y: 0 as Float,
            zoom: 0.5 as Float,
            max_iter: DEFAULT_MAX_ITER,
            ss_factor: 1,
        }
    }
}

fn mandelbrot(c: Complex<Float>, max_iter: Float) -> Float {
    let mut z = Complex::new(0 as Float, 0 as Float);
    let mut n = 0 as Float;
    while z.norm() <= 2.0 && n < max_iter {
        z = z.powf(2.0) + c;
        n += 1.0;
    }
    if n != max_iter {
        return n as Float + 1.0 - z.norm().log10().log10() / (2 as Float).log10();
    }
    n
}

/// Convert a given dimension onto the complex plane the following way:
/// - Map the position -> `[0;1]`
/// - Offset the range by `-0.5` essentially centering it (the center becomes 0) -> `[-0.5;0.5]`
/// - Expand the range -> `[-1;1]`
/// - Scale the range with the given zoom -> `[-1 / zoom;1 / zoom]`
/// - Offset the range with the given center point (move the center) -> `[center + (-1 / zoom);center + (1 / zoom)]`
pub fn map_to_complex_plane(n: Float, max_n: Float, center: Float, zoom: Float) -> Float {
    center + (((n as Float / max_n as Float) - 0.5) * 2 as Float / zoom)
}

pub fn generate_image(width: u32, height: u32, fp: FractalProperties) -> Vec<[u8; 3]> {
    let total_pixels = width * height;

    println!("max_iter: {}", fp.max_iter);
    let regions: Vec<[u8; 3]> = (0..total_pixels)
        .into_par_iter()
        .step_by(PIXEL_CHUNK as usize)
        .map(|start| {
            let end = total_pixels.min(start + PIXEL_CHUNK);
            calculate_region(start..end, width, height, &fp)
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
        pixels.push(calculate_pixel(x as Float, y as Float, max_x, max_y, fp));
    }
    pixels
}

fn calculate_pixel(x: Float, y: Float, max_x: u32, max_y: u32, fp: &FractalProperties) -> [u8; 3] {
    // Supersample the image with the given supersample factor
    let mut vec: Vec<Float> = vec![];
    for u in 0..fp.ss_factor {
        for v in 0..fp.ss_factor {
            let x = x as Float + u as Float / fp.ss_factor as Float;
            let y = y as Float + v as Float / fp.ss_factor as Float;
            let cx = map_to_complex_plane(x as Float, max_x as Float, fp.center_x, fp.zoom);
            let cy = map_to_complex_plane(y as Float, max_y as Float, fp.center_y, fp.zoom);
            let c = Complex::<Float>::new(cx, cy);
            let n = mandelbrot(c, fp.max_iter);
            vec.push(n);
        }
    }

    let n: f64 = vec.iter().sum::<f64>() / vec.len() as f64;

    let mut deg = 0.90 + 10.0 * n;
    while deg > 360.0 {
        deg -= 360.0;
    }

    let hue = Deg(deg);

    let saturation = 0.6;
    let value = if n < fp.max_iter { 1.0f32 } else { 0.0f32 };
    let hsv = Hsv::new(hue, saturation, value);
    let color = Rgb::from_color(&hsv);
    [
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8,
    ]
}

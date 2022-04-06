use std::{f64::consts::PI, ops::Range};

use angular_units::{Deg, Turns};
use num_complex::Complex;
use prisma::{FromColor, Hsv, Rgb};
use rayon::prelude::*;
use rust_decimal::Decimal;

use super::mandelbrot::{map_to_complex_plane, Float, FractalProperties};

const PIXEL_CHUNK: u32 = 10000;

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

fn mandelbrot_bruh(x0: f64, y0: f64, max_iter: Float) -> Float {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut iteration = 0;

    while x * x + y * y < 2.0 * 2.0 && iteration < 180 {
        let xtemp = x * x - y * y + x0;
        y = 2.0 * x * y + y0;
        x = xtemp;
        iteration += 1;
    }
    iteration as Float
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

pub fn generate_image(width: u32, height: u32, fp: FractalProperties) -> Vec<[u8; 3]> {
    let total_pixels = width * height;

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

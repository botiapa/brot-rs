use std::ops::Range;

use angular_units::Turns;
use image::RgbImage;
use num_complex::Complex;
use prisma::{FromColor, Hsv, Rgb};
use rayon::prelude::*;

const MAX_ITER: f32 = 150.0;

const WIDTH: u32 = 3440;
const HEIGHT: u32 = 1440;
const PIXEL_CHUNK: u32 = 10000;

//const WIDTH: u32 = 15360;
//const HEIGHT: u32 = 8640;

const RE_START: f32 = -2.5; // DEFAULT: -2.0
const RE_END: f32 = 1.5; // DEFAULT: 1.0
const IM_START: f32 = -1.5; // DEFAULT: -1.0
const IM_END: f32 = 1.5; // DEFAULT: 1.0

fn mandelbrot(c: Complex<f32>) -> f32 {
    let mut z = Complex::new(0f32,0f32);
    let mut n = 0f32;
    while z.norm() <= 2.0 && n < MAX_ITER {
        z = z*z + c;
        n += 1.0;
    }
    if n != MAX_ITER {
        //return n as f32 + 1.0 - z.norm().log2().log10();
        return n;
    }
    n
}

fn map_to_screen_space(x: f32, max_x: f32, min_y: f32, max_y: f32) -> f32 {
    min_y + (x / max_x) * (max_y-min_y)
}

pub fn generate_image() {
    let mut img = RgbImage::new(WIDTH, HEIGHT);

    let total_pixels = WIDTH*HEIGHT;

    let regions: Vec<[u8; 3]> = (0..total_pixels).into_par_iter().step_by(PIXEL_CHUNK as usize).map(|start| {
        let end = total_pixels.min(start+PIXEL_CHUNK);
        calculate_region(start..end, img.width(), img.height())
    }).flatten().collect();

    img.pixels_mut().zip(regions).for_each(|(old, new)| old.0 = new);

    img.save_with_format("output.bmp", image::ImageFormat::Bmp).unwrap();
}

fn calculate_region(mut pixel_range: Range<u32>, max_x: u32, max_y: u32) -> Vec<[u8; 3]> {
    let mut pixels: Vec<[u8; 3]> = Vec::new();
    //println!("{:?}", a);
    for i in &mut pixel_range {
        let x = i % WIDTH;
        let y = i / WIDTH;
        pixels.push(calculate_pixel(x, y, max_x, max_y));
    }
    pixels
}

fn calculate_pixel(x: u32, y: u32, max_x: u32, max_y: u32) -> [u8; 3] {
    let c = Complex::<f32>::new(
        map_to_screen_space(x as f32, max_x as f32, RE_START, RE_END),
    map_to_screen_space(y as f32, max_y as f32, IM_START, IM_END)
    );
    let n = mandelbrot(c);

    let hue = Turns((n as f32 / MAX_ITER as f32).min(0.9999999));
    
    let saturation = 1.0;
    let value = if n < MAX_ITER { 1.0f32 } else { 0.0f32 };
    let hsv = Hsv::new(hue,saturation,value);
    let color = Rgb::from_color(&hsv);
    [(color.red() * 255.0) as u8,(color.green() * 255.0) as u8,(color.blue() * 255.0) as u8]
}
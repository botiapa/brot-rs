use std::time::Instant;

use angular_units::Turns;
use image::RgbImage;
use num_complex::Complex;
use prisma::{FromColor, Hsv, Rgb};

const MAX_ITER: i32 = 80;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

//const WIDTH: u32 = 15360;
//const HEIGHT: u32 = 8640;

const RE_START: f32 = -2.5; // DEFAULT: -2.0
const RE_END: f32 = 1.5; // DEFAULT: 1.0
const IM_START: f32 = -1.5; // DEFAULT: -1.0
const IM_END: f32 = 1.5; // DEFAULT: 1.0

fn mandelbrot(c: Complex<f32>) -> i32 {
    let mut z = Complex::new(0f32,0f32);
    let mut n = 0i32;
    while z.norm() <= 2.0 && n < MAX_ITER {
        z = z*z + c;
        n += 1;
    }
    if n != MAX_ITER {
        n + 1 - z.norm().log2().log10()
    }
    n
}

fn map_to_screen_space(x: f32, max_x: f32, min_y: f32, max_y: f32) -> f32 {
    min_y + (x / max_x) * (max_y-min_y)
}

fn main() {
    let start = Instant::now();
    let mut img = RgbImage::new(WIDTH, HEIGHT);
    for x in 0..img.width() {
        for y in 0..img.height() {
            let c = Complex::<f32>::new(
                map_to_screen_space(x as f32, img.width() as f32, RE_START, RE_END),
            map_to_screen_space(y as f32, img.height() as f32, IM_START, IM_END)
            );
            let n = mandelbrot(c);

            let hue = Turns((n as f32 / MAX_ITER as f32).min(0.9999999));
            if hue.0 < 0.0 || hue.0 > 360.0 {
                println!("Bruh: {:?}", hue);
            }
            
            let saturation = 1.0;
            let value = if n < MAX_ITER { 1.0f32 } else { 0.0f32 };
            let hsv = Hsv::new(hue,saturation,value);
            let color = Rgb::from_color(&hsv);
            img.put_pixel(x, y, image::Rgb([(color.red() * 255.0) as u8,(color.green() * 255.0) as u8,(color.blue() * 255.0) as u8]));
        }
    }
    img.save_with_format("output.bmp", image::ImageFormat::Bmp).unwrap();
    println!("Elapsed: {}ms", start.elapsed().as_millis());
}

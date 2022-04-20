use angular_units::Deg;
use prisma::{FromColor, Hsv, Rgb};

use super::mandelbrot::FractalProperties;

pub fn calculate_pixel_color(fp: FractalProperties, n: f64) -> [u8; 3] {
    let mut deg = 0.90 + fp.color_offset * n;
    while deg > 360.0 {
        deg -= 360.0;
    }

    let hue = Deg(deg);

    let saturation = fp.color_saturation;
    let value = if (n as f64) < fp.max_iter {
        1.0f32
    } else {
        0.0f32
    };
    let hsv = Hsv::new(hue, saturation, value.into());
    let color = Rgb::from_color(&hsv);

    [
        (color.red() * 255.0) as u8,
        (color.green() * 255.0) as u8,
        (color.blue() * 255.0) as u8,
    ]
}

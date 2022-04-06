use std::time::Instant;

use angular_units::Deg;
use num_complex::Complex64;
use ocl::{prm::Double2, Buffer, Kernel, ProQue};
use prisma::{FromColor, Hsv, Rgb};

use crate::algorithms::mandelbrot::{map_to_complex_plane, Float};

use super::mandelbrot::FractalProperties;

const MANDELBROT_SRC: &str = r#"
__kernel void mandelbrot(__global double2* src_buffer, __global double* buffer, double max_iter) {
    double x0 = src_buffer[get_global_id(0)].x;
    double y0 = src_buffer[get_global_id(0)].y;
    double x = 0.0;
    double y = 0.0;
    long iteration = 0;
    while (x*x + y*y < 2.0*2.0 && iteration < max_iter) {
        double xtemp = x*x - y*y + x0;
        y = 2*x*y + y0;
        x = xtemp;
        iteration++;
    }

    buffer[get_global_id(0)] = iteration;
}
"#;

pub struct OpenCLRenderer {
    pro_que: Option<ProQue>,
    kernel: Option<Kernel>,
    buffer: Option<Buffer<f64>>,
}

impl Default for OpenCLRenderer {
    fn default() -> Self {
        Self {
            pro_que: None,
            kernel: None,
            buffer: None,
        }
    }
}

impl OpenCLRenderer {
    pub fn generate_image(
        &mut self,
        width: u32,
        height: u32,
        fp: FractalProperties,
    ) -> Result<Vec<[u8; 3]>, String> {
        let total_pixels = width * height;

        let mut src_buffer = vec![[0.0f64, 0.0f64]; width as usize * height as usize];
        for x in 0..width {
            for y in 0..height {
                let cx = map_to_complex_plane(x as Float, width as Float, fp.center_x, fp.zoom);
                let cy = map_to_complex_plane(y as Float, height as Float, fp.center_y, fp.zoom);
                src_buffer[y as usize * width as usize + x as usize][0] = cx;
                src_buffer[y as usize * width as usize + x as usize][1] = cy;
            }
        }
        let src_slice: &mut [Double2] = unsafe {
            ::std::slice::from_raw_parts_mut(src_buffer.as_mut_ptr() as *mut _, src_buffer.len())
        };

        // Width and height changed, rebuild needed
        if self.pro_que.is_none()
            || self.pro_que.as_ref().unwrap().dims().to_len() != width as usize * height as usize
        {
            println!("Rebuilt OpenCL pro_que");
            self.build(width, height)?;
        }

        // TODO: The buffer should support complexes directly...
        let src_buffer = Buffer::<Double2>::builder()
            .queue(self.pro_que.as_ref().unwrap().queue().clone())
            .len(width * height)
            .copy_host_slice(src_slice)
            .build()?;

        self.kernel.as_mut().unwrap().set_arg(0i32, &src_buffer)?;
        self.kernel
            .as_mut()
            .unwrap()
            .set_arg(1i32, self.buffer.as_ref().unwrap())?;
        self.kernel.as_mut().unwrap().set_arg(2i32, fp.max_iter)?;

        let start = Instant::now();

        unsafe {
            self.kernel.as_ref().unwrap().enq()?;
        }

        let mut vec = vec![0.0f64; self.buffer.as_ref().unwrap().len()];
        self.buffer.as_ref().unwrap().read(&mut vec).enq()?;
        println!("Elapsed inner: {}ms", start.elapsed().as_millis());

        let img: Vec<[u8; 3]> = vec
            .iter()
            .map(|n| {
                let mut deg = 0.90 + 10.0 * n;
                while deg > 360.0 {
                    deg -= 360.0;
                }

                let hue = Deg(deg);

                let saturation = 0.6;
                let value = if *n < fp.max_iter { 1.0f32 } else { 0.0f32 };
                let hsv = Hsv::new(hue, saturation, value);
                let color = Rgb::from_color(&hsv);
                [
                    (color.red() * 255.0) as u8,
                    (color.green() * 255.0) as u8,
                    (color.blue() * 255.0) as u8,
                ]
            })
            .collect();
        Ok(img)
    }

    fn build(&mut self, width: u32, height: u32) -> Result<(), String> {
        let pro_que = ProQue::builder()
            .src(MANDELBROT_SRC)
            .dims(width * height)
            .build()?;

        self.buffer = Some(pro_que.create_buffer::<f64>()?);
        self.kernel = Some(
            pro_que
                .kernel_builder("mandelbrot")
                .arg_named("src_buffer", None::<&Buffer<Double2>>)
                .arg_named("buffer", None::<&Buffer<f64>>)
                .arg_named("max_iter", 0f64)
                .build()?,
        );
        self.pro_que = Some(pro_que);
        Ok(())
    }
}

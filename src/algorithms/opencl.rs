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
    double iteration = 0.0;
    while (x*x + y*y < 2.0*2.0 && iteration < max_iter) {
        double xtemp = x*x - y*y + x0;
        y = 2*x*y + y0;
        x = xtemp;
        iteration++;
    }
    if(iteration != max_iter) {
        buffer[get_global_id(0)] = iteration + 1.0 - log10(log10(sqrt(pown(x,2) + pown(y,2)))) / log10(2.0);
        return;
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

        let mut src_buffer =
            vec![[0.0f64, 0.0f64]; width as usize * height as usize * fp.ss_factor.pow(2)];
        for x in 0..width {
            for y in 0..height {
                for u in 0..fp.ss_factor {
                    for v in 0..fp.ss_factor {
                        let x = x as Float + u as Float / fp.ss_factor as Float;
                        let y = y as Float + v as Float / fp.ss_factor as Float;
                        let cx =
                            map_to_complex_plane(x as Float, width as Float, fp.center_x, fp.zoom);
                        let cy =
                            map_to_complex_plane(y as Float, height as Float, fp.center_y, fp.zoom);
                        src_buffer[(y as usize + u) * width as usize + (x as usize + u)][0] = cx;
                        src_buffer[(y as usize + v) * width as usize + (x as usize + u)][1] = cy;
                    }
                }
            }
        }
        let src_slice: &mut [Double2] = unsafe {
            ::std::slice::from_raw_parts_mut(src_buffer.as_mut_ptr() as *mut _, src_buffer.len())
        };

        // Width and height changed, rebuild needed
        if self.pro_que.is_none()
            || self.pro_que.as_ref().unwrap().dims().to_len()
                != width as usize * height as usize * fp.ss_factor
        {
            println!("Rebuilt OpenCL pro_que");
            self.build(width, height, fp.ss_factor).expect("bruhhh");
        }

        // TODO: The buffer should support complexes directly...
        let src_buffer = Buffer::<Double2>::builder()
            .queue(self.pro_que.as_ref().unwrap().queue().clone())
            .len(width * height * fp.ss_factor.pow(2) as u32)
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

        let mut img: Vec<[u8; 3]> = vec![[0, 0, 0]; width as usize * height as usize];
        for x in 0..width {
            for y in 0..height {
                let mut sum = 0.0;
                for u in 0..fp.ss_factor {
                    for v in 0..fp.ss_factor {
                        sum += vec[(y as usize + v) * width as usize + (x as usize + u)];
                    }
                }
                let mut n = sum / fp.ss_factor as f64;

                let mut deg = 0.90 + fp.color_offset * (n as f64);
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

                img[y as usize * width as usize + x as usize] = [
                    (color.red() * 255.0) as u8,
                    (color.green() * 255.0) as u8,
                    (color.blue() * 255.0) as u8,
                ]
            }
        }

        Ok(img)
    }

    fn build(&mut self, width: u32, height: u32, ss_scale: usize) -> Result<(), String> {
        let pro_que = ProQue::builder()
            .src(MANDELBROT_SRC)
            .dims(width * height * (ss_scale as u32).pow(2))
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

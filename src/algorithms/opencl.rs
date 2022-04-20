use std::time::Instant;

use ocl::{Buffer, Kernel, ProQue, SpatialDims};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::algorithms::coloring::calculate_pixel_color;

use super::mandelbrot::FractalProperties;

const MANDELBROT_SRC: &str = r#"
struct FractalProperties {
    double center_x;
    double center_y;
    double zoom;
    double max_iter;
    int ss_factor;
    double color_offset;
    double color_saturation;
};  

double map_to_complex_plane(double n, double max_n, double center, double zoom) {
    return center + (((n / max_n) - 0.5) * 2.0 / zoom);
}

__kernel void mandelbrot(struct FractalProperties fp, __global double* buffer) {
    double n = 0.0;
    for(int x_offset = 0; x_offset < fp.ss_factor; x_offset++) {
        for(int y_offset = 0; y_offset < fp.ss_factor; y_offset++) {
            double x0 = (double)get_global_id(0) + (double)x_offset / (double)fp.ss_factor;
            double y0 = (double)get_global_id(1) + (double)y_offset / (double)fp.ss_factor;
            x0 = map_to_complex_plane(x0, get_global_size(0), fp.center_x, fp.zoom);
            y0 = map_to_complex_plane(y0, get_global_size(1), fp.center_y, fp.zoom);
            double x = 0.0;
            double y = 0.0;
            double iteration = 0.0;
            while (x*x + y*y < 2.0*2.0 && iteration < fp.max_iter) {
                double xtemp = x*x - y*y + x0;
                y = 2*x*y + y0;
                x = xtemp;
                iteration += 1.0;
            }

            if(iteration != fp.max_iter)
                n += iteration + 1.0 - log10(log10(sqrt(pown(x,2) + pown(y,2)))) / log10(2.0);
            else
                n += iteration;
        }    
    }
    double avg = n / pown((double)fp.ss_factor, 2);
    buffer[get_global_id(1) * get_global_size(0) + get_global_id(0)] = avg;
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
        let build_timer = Instant::now();
        // Width and height changed, rebuild needed
        if self.pro_que.is_none()
            || self.pro_que.as_ref().unwrap().dims().to_len() != width as usize * height as usize
        {
            println!("Rebuilt OpenCL pro_que");
            self.build(width, height)
                .expect("Failed building opencl pro_que");
        }

        // Set opencl kernel args
        self.kernel.as_mut().unwrap().set_arg(0i32, fp)?;
        self.kernel
            .as_mut()
            .unwrap()
            .set_arg(1i32, self.buffer.as_ref().unwrap())?;
        println!("Elapsed build: {}ms", build_timer.elapsed().as_millis());

        let inner_timer = Instant::now();
        unsafe {
            self.kernel.as_ref().unwrap().enq()?;
        }

        let mut vec = vec![0.0f64; self.buffer.as_ref().unwrap().len()];
        self.buffer.as_ref().unwrap().read(&mut vec).enq()?;
        println!("Elapsed inner: {}ms", inner_timer.elapsed().as_millis());

        let coloring_timer = Instant::now();
        let img: Vec<[u8; 3]> = (0..(width as usize * height as usize))
            .into_par_iter()
            .map(|i| {
                let x = i % width as usize;
                let y = i / width as usize;
                let n = vec[y as usize * width as usize + x as usize];
                calculate_pixel_color(fp, n)
            })
            .collect();
        println!(
            "Elapsed coloring: {}ms",
            coloring_timer.elapsed().as_millis()
        );
        Ok(img)
    }

    fn build(&mut self, width: u32, height: u32) -> Result<(), String> {
        let pro_que = ProQue::builder()
            .src(MANDELBROT_SRC)
            .dims(SpatialDims::Two(width as usize, height as usize))
            .build()?;

        self.buffer = Some(pro_que.create_buffer::<f64>()?);
        self.kernel = Some(
            pro_que
                .kernel_builder("mandelbrot")
                .arg_named("fp", FractalProperties::default())
                .arg_named("buffer", None::<&Buffer<f64>>)
                .build()?,
        );
        self.pro_que = Some(pro_que);
        Ok(())
    }
}

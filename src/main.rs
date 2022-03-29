use std::time::Instant;

mod mandelbrot;

fn main() {
    let start = Instant::now();
    mandelbrot::generate_image();
    println!("Elapsed: {}ms", start.elapsed().as_millis());
}



use std::time::Instant;

mod gui;
mod mandelbrot;

fn main() {
    let start = Instant::now();
    gui::run_gui();
    mandelbrot::generate_image();
    println!("Elapsed: {}ms", start.elapsed().as_millis());
}

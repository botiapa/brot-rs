pub use egui;

mod gui;
mod mandelbrot;
mod renderer;

fn main() {
    gui::run_gui();
}

pub use egui;

mod algorithms;
mod gui;
mod renderer;
mod video_renderer;

fn main() {
    gui::run_gui();
}

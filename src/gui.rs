#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    thread::{self, sleep},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use eframe::{
    egui,
    epi::{App, Frame},
};
use egui::{Color32, TextureHandle};

use crate::{
    mandelbrot::{map_to_complex_plane, Float, FractalProperties},
    renderer::{renderer_thread, RendererMessage},
};

pub fn run_gui() {
    let (renderer_sender, gui_receiver) = renderer_thread();

    let options = eframe::NativeOptions::default();
    let app = MyApp::new(renderer_sender, gui_receiver);
    eframe::run_native("brot-rs", options, Box::new(|_| Box::new(app)));
}

struct MyApp {
    renderer_sender: Sender<RendererMessage>,
    gui_receiver: Receiver<RendererMessage>,
    img_handle: Option<TextureHandle>,
    fp: FractalProperties,
}

impl MyApp {
    fn new(s: Sender<RendererMessage>, r: Receiver<RendererMessage>) -> Self {
        Self {
            renderer_sender: s,
            gui_receiver: r,
            img_handle: None,
            fp: FractalProperties::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("x center: ");
                ui.add(egui::Slider::new(
                    &mut self.fp.center_x,
                    -2 as Float..=2 as Float,
                ));
                ui.label("y center: ");
                ui.add(egui::Slider::new(
                    &mut self.fp.center_y,
                    -2 as Float..=2 as Float,
                ));
                ui.label("zoom: ");
                ui.add(
                    egui::Slider::new(&mut self.fp.zoom, 0 as Float..=100000000000.0 as Float)
                        .logarithmic(true),
                );
                ui.label("max iter: ");
                ui.add(egui::Slider::new(
                    &mut self.fp.max_iter,
                    0 as Float..=1000000 as Float,
                ));
            });
            let (width, height) = (1000, 1000);
            if ui.button("click_me").clicked() {
                self.refresh_img(width, height);
            }
            if let Ok(msg) = self.gui_receiver.try_recv() {
                if let RendererMessage::RenderedImage(img, width, height) = msg {
                    let start = Instant::now();
                    let arr = img
                        .iter()
                        .map(|x| Color32::from_rgb(x[0], x[1], x[2]))
                        .collect::<Vec<Color32>>();
                    let mut color_img =
                        egui::ColorImage::new([width as usize, height as usize], Color32::BLUE);
                    for x in 0..width {
                        for y in 0..height {
                            color_img[(x as usize, y as usize)] = arr[(y * width + x) as usize];
                        }
                    }
                    let txt = ui.ctx().load_texture("0", color_img);
                    self.img_handle.replace(txt);
                    println!("Rendered image in: {}ms", start.elapsed().as_millis());
                    ctx.request_repaint();
                } else {
                    panic!("Received invalid renderer message");
                }
            }

            if self.img_handle.is_none() {
                self.img_handle = Some(ui.ctx().load_texture("0", egui::ColorImage::example()));
                self.refresh_img(width, height);
            }

            let img = ui.add(
                egui::ImageButton::new(
                    self.img_handle.as_ref().unwrap().id(),
                    self.img_handle.as_ref().unwrap().size_vec2(),
                )
                .frame(false),
            );

            if img.clicked() {
                let mut loc = img.interact_pointer_pos().unwrap();
                loc.x -= img.rect.left().max(0f32).min(width as f32);
                loc.y -= img.rect.top().max(0f32).min(height as f32);
                // Calculate clicked point on the complex plane
                self.fp.center_x = map_to_complex_plane(
                    loc.x as Float,
                    width as Float,
                    self.fp.center_x,
                    self.fp.zoom,
                );
                self.fp.center_y = map_to_complex_plane(
                    loc.y as Float,
                    height as Float,
                    self.fp.center_y,
                    self.fp.zoom,
                );
                self.fp.zoom *= 2 as Float;
                println!("{}", self.fp.zoom);
                self.refresh_img(width, height);
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}

impl MyApp {
    fn refresh_img(&self, width: u32, height: u32) {
        self.renderer_sender
            .send(RendererMessage::RenderCommand(
                width as u32,
                height as u32,
                self.fp.clone(),
            ))
            .unwrap();
    }
}

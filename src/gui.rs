#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use self::egui::Ui;
use crossbeam_channel::{Receiver, Sender};
use eframe::{
    egui,
    epaint::ImageDelta,
    epi::{App, Frame},
};
use egui::{Color32, ColorImage, ImageData, Sense, TextureHandle, TextureId};

use crate::{
    mandelbrot::{FractalProperties, IM_END, IM_START, RE_END, RE_START},
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
            /*ui.heading("My egui Application");
                        ui.horizontal(|ui| {
                            ui.label("Your name: ");
                            ui.text_edit_singleline(&mut self.name);
                        });
                        ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
                        if ui.button("Click each year").clicked() {
                            self.age += 1;
                        }
                        ui.label(format!("Hello '{}', age {}", self.name, self.age));
            */
            ui.horizontal(|ui| {
                ui.label("im start: ");
                ui.add(egui::Slider::new(&mut self.fp.im_start, -1.5..=1.5));
                ui.label("im end: ");
                ui.add(egui::Slider::new(&mut self.fp.im_end, 0.0..=1.5));
                ui.label("re start: ");
                ui.add(egui::Slider::new(&mut self.fp.re_start, -2.5..=2.0));
                ui.label("re end: ");
                ui.add(egui::Slider::new(&mut self.fp.re_end, 0.0..=1.5));
                ui.label("scale: ");
                ui.add(egui::Slider::new(&mut self.fp.scale, -10.0..=10.0).logarithmic(true));
            });
            let available = ui.available_size();

            if ui.button("click_me").clicked() {
                self.renderer_sender
                    .send(RendererMessage::RenderCommand(
                        1280 as u32,
                        720 as u32,
                        self.fp.clone(),
                    ))
                    .unwrap();
            }
            if let Ok(msg) = self.gui_receiver.try_recv() {
                println!("Received");
                if let RendererMessage::RenderedImage(img, width, height) = msg {
                    println!("Received rendered image");
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
                    println!("Replaced img handle");
                } else {
                    panic!("Received invalid renderer message");
                }
            }

            if self.img_handle.is_none() {
                self.img_handle = Some(ui.ctx().load_texture("0", egui::ColorImage::example()));
                println!("Example img");
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
                loc.x -= img
                    .rect
                    .left()
                    .max(0f32)
                    .min(self.img_handle.as_ref().unwrap().size()[0] as f32);
                loc.y -= img
                    .rect
                    .top()
                    .max(0f32)
                    .min(self.img_handle.as_ref().unwrap().size()[1] as f32);
                let center = (
                    (loc.x / self.img_handle.as_ref().unwrap().size()[0] as f32) - 0.5,
                    (loc.y / self.img_handle.as_ref().unwrap().size()[1] as f32) - 0.5,
                );
                println!(
                    "loc: {:?} x:{} y:{}",
                    center,
                    img.rect.left(),
                    img.rect.top()
                );
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
    }
}

impl MyApp {
    fn custom_painting(&self, ui: &mut Ui) {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::splat(300.0), egui::Sense::drag());

        // Clone locals so we can move them into the paint callback:

        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(move |_info, render_ctx| {
                /*if let Some(painter) = render_ctx.downcast_mut::<egui_glow::Painter>() {
                    let img = ColorImage::example();
                    let img = ImageData::Color(img);
                    let delta = ImageDelta::full(img);
                    painter.set_texture(TextureId::User(0), &delta);
                } else {
                    eprintln!("Can't do custom painting because we are not using a glow context");
                }*/
            }),
        };
        ui.painter().add(callback);
    }
}

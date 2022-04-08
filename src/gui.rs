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
use image::{GenericImage, ImageBuffer, Pixel};

use crate::{
    algorithms::mandelbrot::{map_to_complex_plane, AlgorithmType, Float, FractalProperties},
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
    img_data: Option<Vec<[u8; 3]>>,
    fp: FractalProperties,
    video_render: Option<VideoRender>,
    render_algorithm: AlgorithmType,
}

struct VideoRender {
    current_frame: u32,
    zoom_speed: Float,
    max_zoom_speed: Float,
    max_zoom: Float,
}

impl Default for VideoRender {
    fn default() -> Self {
        Self {
            current_frame: 0,
            zoom_speed: 0.0005,
            max_zoom_speed: 10000000.0,
            max_zoom: 10000000000.0,
        }
    }
}

impl MyApp {
    fn new(s: Sender<RendererMessage>, r: Receiver<RendererMessage>) -> Self {
        Self {
            renderer_sender: s,
            gui_receiver: r,
            img_handle: None,
            img_data: None,
            fp: FractalProperties::default(),
            video_render: None,
            render_algorithm: AlgorithmType::NaiveCPU,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (width, height) = (1280, 720);

            if self.video_render.is_none() {
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
                        egui::Slider::new(
                            &mut self.fp.zoom,
                            0 as Float..=10000000000000.0 as Float,
                        )
                        .logarithmic(true),
                    );
                    ui.label("max iter: ");
                    ui.add(egui::Slider::new(
                        &mut self.fp.max_iter,
                        0 as Float..=1000000000 as Float,
                    ));
                    ui.label("ss factor: ");
                    ui.add(egui::Slider::new(&mut self.fp.ss_factor, 1..=32));

                    let alg_was = self.render_algorithm.clone();
                    egui::ComboBox::from_label("Renderer")
                        .selected_text(format!("{:?}", self.render_algorithm))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.render_algorithm,
                                AlgorithmType::NaiveCPU,
                                "NaiveCPU",
                            );
                            #[cfg(feature = "opencl")]
                            ui.selectable_value(
                                &mut self.render_algorithm,
                                AlgorithmType::OpenCL,
                                "OpenCL",
                            );
                        });

                    if alg_was != self.render_algorithm {
                        self.refresh_img(width, height);
                    }
                });
            }

            ui.horizontal(|ui| {
                if self.video_render.is_none() && ui.button("Render").clicked() {
                    self.refresh_img(width, height);
                }

                if self.video_render.is_none()
                    && ui.button("Save image").clicked()
                    && self.img_data.is_some()
                {
                    self.save_img(width, height, "screenshot.bmp");
                }

                if ui.button("Render video").clicked() {
                    if self.video_render.is_none() {
                        self.video_render = Some(VideoRender::default());
                        self.refresh_img(width, height);
                    } else {
                        self.video_render = None;
                    }
                }

                ui.label("Color offset:");
                ui.add(egui::Slider::new(&mut self.fp.color_offset, 1.0..=360.0));
                ui.label("Saturation:");
                ui.add(egui::Slider::new(&mut self.fp.color_saturation, 0.1..=1.0));
            });

            if let Ok(msg) = self.gui_receiver.try_recv() {
                if let RendererMessage::RenderedImage(img, width, height) = msg {
                    if self.video_render.is_none() {
                        let start = Instant::now();
                        self.img_data = Some(img.clone());
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
                        let video_render =
                            self.video_render.as_ref().expect("Invalid state reached");
                        self.img_data = Some(img.clone());
                        self.save_img(
                            width,
                            height,
                            &format!("O:\\tmp\\video{}.bmp", video_render.current_frame),
                        );
                        // Video is finished
                        if self.fp.zoom >= video_render.max_zoom {
                            self.video_render = None;
                        } else {
                            self.advance_video_frame(width, height);
                        }
                    }
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
                self.refresh_img(width, height);
            }
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        // If we're currently rendering a video, then always repaint.
        if self.video_render.is_some() {
            sleep(Duration::from_millis(100));
            ctx.request_repaint();
        }
    }
}

impl MyApp {
    fn refresh_img(&self, width: u32, height: u32) {
        self.renderer_sender
            .send(RendererMessage::RenderCommand(
                width as u32,
                height as u32,
                self.render_algorithm.clone(),
                self.fp.clone(),
            ))
            .unwrap();
    }

    fn save_img(&self, width: u32, height: u32, filename: &str) {
        let data = self.img_data.as_ref().unwrap();
        let img = ImageBuffer::from_fn(width, height, |x, y| {
            image::Rgb(data[(y * width + x) as usize])
        });
        img.save(filename).unwrap();
    }

    fn advance_video_frame(&mut self, width: u32, height: u32) {
        let video = self.video_render.as_mut().unwrap();
        video.current_frame += 1;
        self.fp.zoom += video.zoom_speed;
        video.zoom_speed = (video.zoom_speed * 1.1).min(video.max_zoom_speed);
        self.refresh_img(width, height);
    }
}

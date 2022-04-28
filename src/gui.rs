#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use eframe::{
    egui,
    epi::{App, Frame},
};
use egui::{Color32, TextureHandle};
use fontdue::{
    layout::{Layout, LayoutSettings, TextStyle},
    Font,
};
use image::{imageops::FilterType, ImageBuffer};

use crate::{
    algorithms::mandelbrot::{map_to_complex_plane, AlgorithmType, Float, FractalProperties},
    renderer::{renderer_thread, RendererMessage},
};

/// Split up every rendered frame into scaled sub-frames
const FRAMES_PER_IMG: usize = 1;

const FONT_DATA: &[u8] = include_bytes!("../font.ttf");

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
    max_zoom: Float,
    render_started: Instant,
    font: Font,
    layout: Layout,
}

impl Default for VideoRender {
    fn default() -> Self {
        Self {
            current_frame: 0,
            max_zoom: 10000000000.0,
            render_started: Instant::now(),
            font: fontdue::Font::from_bytes(FONT_DATA, fontdue::FontSettings::default())
                .expect("Failed loading in font"),
            layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown),
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
            #[cfg(not(feature = "opencl"))]
            render_algorithm: AlgorithmType::NaiveCPU,
            #[cfg(feature = "opencl")]
            render_algorithm: AlgorithmType::OpenCL,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (width, height) = (7680, 4320);
            let (scaled_width, scaled_height) = (1920, 1080);

            assert!(width >= scaled_width && height >= scaled_height);

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
                        self.video_render = Some(VideoRender {
                            max_zoom: self.fp.zoom,
                            ..VideoRender::default()
                        });
                        self.fp.zoom = 0.5;
                        self.refresh_img(width, height);
                    } else {
                        self.video_render = None;
                    }
                }
                if self.video_render.is_none() {
                    ui.label("Color offset:");
                    ui.add(egui::Slider::new(&mut self.fp.color_offset, 1.0..=360.0));
                    ui.label("Saturation:");
                    ui.add(egui::Slider::new(&mut self.fp.color_saturation, 0.1..=1.0));
                }
            });

            if let Ok(msg) = self.gui_receiver.try_recv() {
                if let RendererMessage::RenderedImage(img_data, width, height) = msg {
                    self.img_data = Some(img_data);
                    if let Some(vr) = &self.video_render {
                        // Video is finished
                        if self.fp.zoom >= vr.max_zoom {
                            let vr = self.video_render.take().unwrap();
                            println!(
                                "Finished rendering the video in: {}s",
                                vr.render_started.elapsed().as_secs()
                            );
                        } else {
                            self.advance_video_frame(width, height);
                        }
                    } else {
                        let rendering_timer = Instant::now();

                        let img = ImageBuffer::from_fn(width, height, |x, y| {
                            image::Rgb(self.img_data.as_ref().unwrap()[(y * width + x) as usize])
                        });

                        let resizing_timer = Instant::now();
                        // Skip resizing if not needed
                        let resized = if width == scaled_width && height == scaled_height {
                            img
                        } else {
                            image::imageops::resize(
                                &img,
                                scaled_width,
                                scaled_height,
                                FilterType::Triangle,
                            )
                        };
                        println!("Resizing took: {}ms", resizing_timer.elapsed().as_millis());

                        let mut color_img = egui::ColorImage::new(
                            [resized.width() as usize, resized.height() as usize],
                            Color32::BLUE,
                        );

                        for x in 0..resized.width() {
                            for y in 0..resized.height() {
                                let res_pix = resized[(x, y)].0;
                                color_img[(x as usize, y as usize)] =
                                    Color32::from_rgb(res_pix[0], res_pix[1], res_pix[2]);
                            }
                        }

                        let txt = ui.ctx().load_texture("0", color_img);
                        self.img_handle.replace(txt);
                        println!(
                            "Rendered image in: {}ms",
                            rendering_timer.elapsed().as_millis()
                        );
                        ctx.request_repaint();
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
                let displayed_img_size = self.img_handle.as_ref().unwrap().size_vec2();
                // Calculate clicked point on the complex plane
                self.fp.center_x = map_to_complex_plane(
                    loc.x as Float,
                    displayed_img_size.x as Float,
                    self.fp.center_x,
                    self.fp.zoom,
                );
                self.fp.center_y = map_to_complex_plane(
                    loc.y as Float,
                    displayed_img_size.y as Float,
                    self.fp.center_y,
                    self.fp.zoom,
                );
                self.fp.zoom *= 2 as Float;
                self.refresh_img(width, height);
            }
        });

        // If we're currently rendering a video, then always repaint.
        if self.video_render.is_some() {
            sleep(Duration::from_millis(100));
            ctx.request_repaint();
        }
    }
}

impl MyApp {
    /// Send a rendering request to the rendering backend.
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
}

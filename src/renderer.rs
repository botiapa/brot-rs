use std::{thread, time::Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::algorithms::mandelbrot::{AlgorithmType, FractalProperties};

pub enum RendererMessage {
    RenderCommand(u32, u32, AlgorithmType, FractalProperties),
    RenderedImage(Vec<[u8; 3]>, u32, u32),
}

pub fn renderer_thread() -> (Sender<RendererMessage>, Receiver<RendererMessage>) {
    let (s1, r1) = unbounded();
    let (s2, r2) = unbounded();

    thread::spawn(move || {
        renderer_loop(s1, r2);
    });
    (s2, r1)
}

fn renderer_loop(
    gui_sender: Sender<RendererMessage>,
    renderer_receiver: Receiver<RendererMessage>,
) {
    loop {
        for cmd in &renderer_receiver {
            if let RendererMessage::RenderCommand(width, height, algorithm, fp) = cmd {
                let start = Instant::now();
                let img = match algorithm {
                    AlgorithmType::NaiveCPU => NaiveCPU::generate_image(width, height, fp),
                    AlgorithmType::OpenCL => OpenCL::generate_image(width, height, fp),
                };
                gui_sender
                    .send(RendererMessage::RenderedImage(img, width, height))
                    .unwrap();
                println!(
                    "Generated and sent image in: {}ms",
                    start.elapsed().as_millis()
                );
            }
        }
    }
}

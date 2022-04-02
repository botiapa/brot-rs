use std::{thread, time::Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::mandelbrot::{generate_image, FractalProperties};

pub enum RendererMessage {
    RenderCommand(u32, u32, FractalProperties),
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
            if let RendererMessage::RenderCommand(width, height, fp) = cmd {
                let start = Instant::now();
                gui_sender
                    .send(RendererMessage::RenderedImage(
                        generate_image(width, height, fp),
                        width,
                        height,
                    ))
                    .unwrap();
                println!(
                    "Generated and sent image in: {}ms",
                    start.elapsed().as_millis()
                );
            }
        }
    }
}

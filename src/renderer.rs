use std::{thread, time::Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::algorithms::{
    mandelbrot::{AlgorithmType, FractalProperties},
    opencl::OpenCLRenderer,
    *,
};

pub enum RendererMessage {
    RenderCommand(u32, u32, AlgorithmType, FractalProperties),
    RenderedImage(Vec<[u8; 3]>, u32, u32),
}

pub fn renderer_thread() -> (Sender<RendererMessage>, Receiver<RendererMessage>) {
    let (s1, r1) = unbounded();
    let (s2, r2) = unbounded();

    thread::spawn(move || {
        let r = RendererThread::default();
        r.renderer_loop(s1, r2);
    });
    (s2, r1)
}

struct RendererThread {
    #[cfg(feature = "opencl")]
    opencl_renderer: OpenCLRenderer,
}

impl Default for RendererThread {
    fn default() -> Self {
        Self {
            #[cfg(feature = "opencl")]
            opencl_renderer: Default::default(),
        }
    }
}

impl RendererThread {
    fn renderer_loop(
        mut self,
        gui_sender: Sender<RendererMessage>,
        renderer_receiver: Receiver<RendererMessage>,
    ) {
        loop {
            for cmd in &renderer_receiver {
                if let RendererMessage::RenderCommand(width, height, algorithm, fp) = cmd {
                    let start = Instant::now();
                    let img = match algorithm {
                        AlgorithmType::NaiveCPU => naive_cpu::generate_image(width, height, fp),
                        #[cfg(feature = "opencl")]
                        AlgorithmType::OpenCL => self
                            .opencl_renderer
                            .generate_image(width, height, fp)
                            .unwrap(),
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
}

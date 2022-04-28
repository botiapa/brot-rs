const FONT_DATA: &[u8] = include_bytes!("../font.ttf");

pub struct VideoRender {
    pub current_frame: u32,
    pub max_zoom: f64,
    pub render_started: Instant,
    pub font: Font,
    pub layout: Layout,
    pub rendering_finished: Option<Instant>,
}

impl Default for VideoRender {
    fn default() -> Self {
        Self {
            current_frame: Default::default(),
            max_zoom: Default::default(),
            render_started: Instant::now(),
            font: Font::from_bytes(FONT_DATA, FontSettings::default()).unwrap(),
            layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown),
            rendering_finished: None,
        }
    }
}

impl VideoRender {
    pub fn advance_video_frame(
        &mut self,
        zoom: &mut f64,
        img_data: &Vec<[u8; 3]>,
        width: u32,
        height: u32,
    ) {
        let mut img = ImageBuffer::from_fn(width, height, |x, y| {
            image::Rgb(img_data[(y * width + x) as usize])
        });

        let text_timer = Instant::now();
        // TEXT
        const FONT_SIZE: f32 = 50.0;
        self.layout.append(
            &[&self.font],
            &TextStyle::new(&zoom.round().to_string(), FONT_SIZE, 0),
        );
        for glyph in self.layout.glyphs() {
            if glyph.char_data.rasterize() {
                let (metrics, bmp) = self.font.rasterize(glyph.parent, FONT_SIZE);
                for (i, p) in bmp.iter().enumerate() {
                    // Skip black pixels
                    if p == &255 {
                        let x = (i % metrics.width) + glyph.x as usize + 30;
                        let y = (i / metrics.width) + glyph.y as usize + 30;
                        img[(x as u32, y as u32)] = image::Rgb([*p, *p, *p]);
                    }
                }
            }
        }
        self.layout.reset(&LayoutSettings::default());
        println!(
            "Rendering text took: {}ms",
            text_timer.elapsed().as_millis()
        );

        img.save(&format!("O:\\tmp\\video{}.bmp", self.current_frame))
            .expect("Failed saving video frame");
        self.current_frame += 1;

        if *zoom >= self.max_zoom {
            self.rendering_finished = Some(Instant::now());
        }

        *zoom *= 1.05;
    }

    /// Has the rendering been finished
    pub fn finished(&self) -> bool {
        self.rendering_finished.is_some()
    }

    pub fn render_time(&self) -> Duration {
        self.render_started - self.rendering_finished.unwrap()
    }
}

// Frames are broken down into sub-frames. The sub-frames are not perfect frames, and take a lot of CPU cycles.
// Currently it's not worth it to use this kind of rendering, since GPU is so much faster, but it might make
// sense with arbitrary floating point numbers.
/*fn calculate_sub_frame() {
    let desired_size = self.img_handle.clone().unwrap().size_vec2();
    let next_zoom = self.fp.zoom * 2f64;
    let zoom_diff = self.fp.zoom / next_zoom;
    for i in 0..FRAMES_PER_IMG {
        let curr_scale = 1f64 - (zoom_diff / FRAMES_PER_IMG as f64) * i as f64;
        let new_width = width as f64 * curr_scale;
        let new_height = height as f64 * curr_scale;

        let x_offs = ((width as f32 - new_width as f32) / 2.0).ceil() as u32;
        let y_offs = ((height as f32 - new_height as f32) / 2.0).ceil() as u32;
        let cropped = image::imageops::crop_imm(
            &img,
            x_offs,
            y_offs,
            new_width.floor() as u32,
            new_height.floor() as u32,
        )
        .to_image();
        let mut resized = image::imageops::resize(
            &cropped,
            desired_size.x as u32,
            desired_size.y as u32,
            FilterType::Lanczos3,
        );

        // TEXT
        let curr_zoom =
            self.fp.zoom + (next_zoom - self.fp.zoom) / FRAMES_PER_IMG as f64 * i as f64;
        const FONT_SIZE: f32 = 50.0;
        layout.append(
            &[&vr.font],
            &TextStyle::new(&curr_zoom.round().to_string(), FONT_SIZE, 0),
        );
        for glyph in layout.glyphs() {
            if glyph.char_data.rasterize() {
                let (metrics, bmp) = vr.font.rasterize(glyph.parent, FONT_SIZE);
                for (i, p) in bmp.iter().enumerate() {
                    // Skip black pixels
                    if p == &255 {
                        let x = (i % metrics.width) + glyph.x as usize + 30;
                        let y = (i / metrics.width) + glyph.y as usize + 30;
                        resized[(x as u32, y as u32)] = image::Rgb([*p, *p, *p]);
                    }
                }
            }
        }
        layout.reset(&LayoutSettings::default());

        resized
            .save(&format!("O:\\tmp\\video{}.bmp", vr.current_frame))
            .expect("Failed saving video frame");
        vr.current_frame += 1;
    }

    self.fp.zoom = next_zoom;
}*/

use std::time::{Duration, Instant};

use fontdue::{
    layout::{Layout, LayoutSettings, TextStyle},
    Font, FontSettings,
};
use image::ImageBuffer;

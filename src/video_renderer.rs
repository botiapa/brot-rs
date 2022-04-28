struct VideoRender {}

impl VideoRender {
    fn advance_video_frame(&mut self, width: u32, height: u32) {
        let mut img = ImageBuffer::from_fn(width, height, |x, y| {
            image::Rgb(self.img_data.as_ref().unwrap()[(y * width + x) as usize])
        });

        let text_timer = Instant::now();
        // TEXT
        const FONT_SIZE: f32 = 50.0;
        vr.layout.append(
            &[&vr.font],
            &TextStyle::new(&self.fp.zoom.round().to_string(), FONT_SIZE, 0),
        );
        for glyph in vr.layout.glyphs() {
            if glyph.char_data.rasterize() {
                let (metrics, bmp) = vr.font.rasterize(glyph.parent, FONT_SIZE);
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
        vr.layout.reset(&LayoutSettings::default());
        println!(
            "Rendering text took: {}ms",
            text_timer.elapsed().as_millis()
        );

        img.save(&format!("O:\\tmp\\video{}.bmp", vr.current_frame))
            .expect("Failed saving video frame");
        vr.current_frame += 1;

        self.fp.zoom *= 1.05;

        self.refresh_img(width, height);
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

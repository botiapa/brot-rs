pub fn generate_image(width: u32, height: u32, fp: FractalProperties) -> Vec<[u8; 3]> {
    let total_pixels = width * height;

    println!("max_iter: {}", fp.max_iter);
    let regions: Vec<[u8; 3]> = (0..total_pixels)
        .into_par_iter()
        .step_by(PIXEL_CHUNK as usize)
        .map(|start| {
            let end = total_pixels.min(start + PIXEL_CHUNK);
            calculate_region(start..end, width, height, &fp)
        })
        .flatten()
        .collect();
    regions
}

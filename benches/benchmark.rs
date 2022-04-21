use brot_rs::algorithms::mandelbrot::map_to_complex_plane;
use brot_rs::algorithms::naive_cpu::{generate_image, mandelbrot};
use brot_rs::algorithms::{coloring::calculate_pixel_color, mandelbrot::FractalProperties};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use num_complex::Complex;

fn cpu_benchmark(c: &mut Criterion) {
    let fp = FractalProperties::default();
    let mut group = c.benchmark_group("naive_cpu");
    for dims in [
        ("360p", 200, 640, 360),
        ("720p", 100, 1280, 720),
        ("1080p", 30, 1920, 1080),
        ("4k", 15, 3840, 2160),
    ] {
        let (name, sample_size, width, height) = dims;
        group.throughput(Throughput::Elements(width * height));
        group.sample_size(sample_size);
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(width, height, fp),
            |b, (width, height, fp)| {
                b.iter(|| {
                    generate_image(*width as u32, *height as u32, *fp);
                });
            },
        );
    }
    group.finish();
}

fn coloring_benchmark(c: &mut Criterion) {
    let fp = FractalProperties::default();
    let (width, height) = (1920, 1080);
    let samples = (0..width * height)
        .map(|i| {
            let x = i % width;
            let y = i % width;
            let cx = map_to_complex_plane(x as f64, width as f64, fp.center_x, fp.zoom);
            let cy = map_to_complex_plane(y as f64, height as f64, fp.center_y, fp.zoom);
            let c = Complex::<f64>::new(cx, cy);
            mandelbrot(c, fp.max_iter)
        })
        .collect::<Vec<f64>>();
    c.bench_with_input(
        BenchmarkId::new("coloring", "1080p"),
        &(fp, samples),
        |b, (fp, samples)| {
            b.iter(|| {
                for &sample in samples {
                    calculate_pixel_color(*fp, sample);
                }
            });
        },
    );
}

criterion_group!(benches, cpu_benchmark, coloring_benchmark);
criterion_main!(benches);

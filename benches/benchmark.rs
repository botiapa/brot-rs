use brot_rs::algorithms::mandelbrot::FractalProperties;
use brot_rs::algorithms::naive_cpu::generate_image;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

fn cpu_benchmark(c: &mut Criterion) {
    let fp = FractalProperties::default();
    let mut group = c.benchmark_group("naive_cpu");
    for dims in [
        ("360p", 640, 360),
        ("720p", 1280, 720),
        ("1080p", 1920, 1080),
        ("4k", 3840, 2160),
    ] {
        let (name, width, height) = dims;
        group.throughput(Throughput::Elements(width * height));
        group.sample_size(20);
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

criterion_group!(benches, cpu_benchmark);
criterion_main!(benches);

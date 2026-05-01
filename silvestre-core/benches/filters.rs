use criterion::{black_box, criterion_group, criterion_main, Criterion};
use silvestre_core::{
    effects::{BrightnessFilter, GrayscaleFilter, InvertFilter},
    filters::Filter,
    transform::{ResizeFilter, Interpolation, MirrorFilter, MirrorMode},
    ColorSpace, SilvestreImage,
};

// Helper to create test images of various sizes
fn create_test_image(width: u32, height: u32, color_space: ColorSpace) -> SilvestreImage {
    let channels = color_space.channels();
    let len = (width as usize) * (height as usize) * channels;
    let pixels = vec![128u8; len]; // Neutral gray
    SilvestreImage::new(pixels, width, height, color_space).unwrap()
}

fn bench_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("effects");

    let img_100x100 = black_box(create_test_image(100, 100, ColorSpace::Rgb));
    let img_512x512 = black_box(create_test_image(512, 512, ColorSpace::Rgb));

    group.bench_function("brightness_100x100", |b| {
        b.iter(|| {
            let filter = BrightnessFilter::new(50);
            filter.apply(&img_100x100)
        });
    });

    group.bench_function("brightness_512x512", |b| {
        b.iter(|| {
            let filter = BrightnessFilter::new(50);
            filter.apply(&img_512x512)
        });
    });

    group.bench_function("grayscale_100x100", |b| {
        b.iter(|| GrayscaleFilter.apply(&img_100x100))
    });

    group.bench_function("grayscale_512x512", |b| {
        b.iter(|| GrayscaleFilter.apply(&img_512x512))
    });

    group.bench_function("invert_100x100", |b| {
        b.iter(|| InvertFilter.apply(&img_100x100))
    });

    group.bench_function("invert_512x512", |b| {
        b.iter(|| InvertFilter.apply(&img_512x512))
    });

    group.finish();
}

fn bench_transforms(c: &mut Criterion) {
    let mut group = c.benchmark_group("transforms");

    let img_100x100 = black_box(create_test_image(100, 100, ColorSpace::Grayscale));
    let img_512x512 = black_box(create_test_image(512, 512, ColorSpace::Grayscale));

    // Mirror benchmarks
    group.bench_function("mirror_horizontal_100x100", |b| {
        b.iter(|| MirrorFilter::new(MirrorMode::Horizontal).apply(&img_100x100))
    });

    group.bench_function("mirror_horizontal_512x512", |b| {
        b.iter(|| MirrorFilter::new(MirrorMode::Horizontal).apply(&img_512x512))
    });

    group.bench_function("mirror_vertical_100x100", |b| {
        b.iter(|| MirrorFilter::new(MirrorMode::Vertical).apply(&img_100x100))
    });

    group.bench_function("mirror_both_512x512", |b| {
        b.iter(|| MirrorFilter::new(MirrorMode::Both).apply(&img_512x512))
    });

    // Resize benchmarks
    group.bench_function("resize_nearest_neighbor_100x100_to_50x50", |b| {
        b.iter(|| {
            ResizeFilter::new(50, 50, Interpolation::NearestNeighbor).apply(&img_100x100)
        })
    });

    group.bench_function("resize_bilinear_100x100_to_200x200", |b| {
        b.iter(|| {
            ResizeFilter::new(200, 200, Interpolation::Bilinear).apply(&img_100x100)
        })
    });

    group.bench_function("resize_bilinear_512x512_to_256x256", |b| {
        b.iter(|| {
            ResizeFilter::new(256, 256, Interpolation::Bilinear).apply(&img_512x512)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_effects, bench_transforms);
criterion_main!(benches);

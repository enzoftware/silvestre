use criterion::{criterion_group, criterion_main, Criterion};
use silvestre_core::{
    effects::{BrightnessFilter, GrayscaleFilter, InvertFilter},
    filters::Filter,
    simd,
    transform::{Interpolation, MirrorFilter, MirrorMode, ResizeFilter},
    ColorSpace, SilvestreImage,
};
use std::hint::black_box;

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

fn bench_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_scalar");

    let size = 512 * 512 * 3; // 512x512 RGB = 786,432 bytes
    let src = black_box(vec![120u8; size]);
    let mut dst = vec![0u8; size];

    // Invert benchmark
    group.bench_function("invert_scalar_512x512", |b| {
        b.iter(|| {
            simd::scalar::invert(&src, &mut dst, 3);
            black_box(&dst);
        });
    });

    group.bench_function("invert_simd_512x512", |b| {
        b.iter(|| {
            simd::invert(&src, &mut dst, 3);
            black_box(&dst);
        });
    });

    // Brightness add benchmark
    group.bench_function("brightness_add_scalar_512x512", |b| {
        b.iter(|| {
            simd::scalar::brightness_add(&src, &mut dst, 40, 3);
            black_box(&dst);
        });
    });

    group.bench_function("brightness_add_simd_512x512", |b| {
        b.iter(|| {
            simd::brightness_add(&src, &mut dst, 40, 3);
            black_box(&dst);
        });
    });

    // Grayscale conversion benchmark
    let mut dst_gray = vec![0u8; 512 * 512];
    group.bench_function("grayscale_scalar_512x512", |b| {
        b.iter(|| {
            simd::scalar::grayscale_rgb(&src, &mut dst_gray);
            black_box(&dst_gray);
        });
    });

    group.bench_function("grayscale_simd_512x512", |b| {
        b.iter(|| {
            simd::grayscale_rgb(&src, &mut dst_gray);
            black_box(&dst_gray);
        });
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
        b.iter(|| ResizeFilter::new(50, 50, Interpolation::NearestNeighbor).apply(&img_100x100))
    });

    group.bench_function("resize_bilinear_100x100_to_200x200", |b| {
        b.iter(|| ResizeFilter::new(200, 200, Interpolation::Bilinear).apply(&img_100x100))
    });

    group.bench_function("resize_bilinear_512x512_to_256x256", |b| {
        b.iter(|| ResizeFilter::new(256, 256, Interpolation::Bilinear).apply(&img_512x512))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_effects,
    bench_simd_vs_scalar,
    bench_transforms
);
criterion_main!(benches);

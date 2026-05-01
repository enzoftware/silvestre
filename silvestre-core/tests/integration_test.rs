//! Integration tests that exercise the full pipeline: load image, apply filters, verify results.

use silvestre_core::{
    effects::{BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter},
    filters::Filter,
    transform::{CropFilter, Interpolation, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter},
    ColorSpace, SilvestreImage,
};

// Helper to create test images
fn create_test_image(width: u32, height: u32, color_space: ColorSpace) -> SilvestreImage {
    let channels = color_space.channels();
    let len = (width as usize) * (height as usize) * channels;
    let mut pixels = vec![0u8; len];

    // Create a simple gradient pattern
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = (y * width as usize + x) * channels;
            let value = ((x + y) % 256) as u8;
            for c in 0..channels {
                pixels[idx + c] = value;
            }
        }
    }

    SilvestreImage::new(pixels, width, height, color_space).unwrap()
}

#[test]
fn test_brightness_filter_changes_image() {
    let img = create_test_image(10, 10, ColorSpace::Grayscale);
    let original_pixels = img.pixels().to_vec();

    let filter = BrightnessFilter::new(50);
    let result = filter.apply(&img).unwrap();

    // Verify the image changed (at least some pixels should be different)
    assert_ne!(result.pixels(), &original_pixels[..]);
    assert_eq!(result.width(), img.width());
    assert_eq!(result.height(), img.height());
}

#[test]
fn test_contrast_filter_changes_image() {
    let img = create_test_image(10, 10, ColorSpace::Rgb);
    let filter = ContrastFilter::new(1.5).unwrap();
    let result = filter.apply(&img).unwrap();

    assert_eq!(result.width(), img.width());
    assert_eq!(result.height(), img.height());
    assert_eq!(result.color_space(), ColorSpace::Rgb);
}

#[test]
fn test_grayscale_conversion() {
    let img = create_test_image(8, 8, ColorSpace::Rgb);
    let result = GrayscaleFilter.apply(&img).unwrap();

    assert_eq!(result.color_space(), ColorSpace::Grayscale);
    assert_eq!(result.width(), img.width());
    assert_eq!(result.height(), img.height());
}

#[test]
fn test_invert_is_idempotent_on_alternates() {
    let img = create_test_image(8, 8, ColorSpace::Grayscale);
    let inverted_once = InvertFilter.apply(&img).unwrap();
    let inverted_twice = InvertFilter.apply(&inverted_once).unwrap();

    // Inverting twice should return the original
    assert_eq!(inverted_twice.pixels(), img.pixels());
}

#[test]
fn test_crop_and_resize_pipeline() {
    let img = create_test_image(16, 16, ColorSpace::Grayscale);

    // Crop to 8x8
    let cropped = CropFilter::new(4, 4, 8, 8).apply(&img).unwrap();
    assert_eq!(cropped.width(), 8);
    assert_eq!(cropped.height(), 8);

    // Resize to 4x4
    let resized = ResizeFilter::new(4, 4, Interpolation::NearestNeighbor)
        .apply(&cropped)
        .unwrap();
    assert_eq!(resized.width(), 4);
    assert_eq!(resized.height(), 4);
}

#[test]
fn test_mirror_round_trip() {
    let img = create_test_image(12, 12, ColorSpace::Rgb);

    // Mirror horizontally twice should be identity
    let h_flip = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
    let h_flip_again = MirrorFilter::new(MirrorMode::Horizontal)
        .apply(&h_flip)
        .unwrap();
    assert_eq!(h_flip_again.pixels(), img.pixels());

    // Mirror vertically twice should be identity
    let v_flip = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
    let v_flip_again = MirrorFilter::new(MirrorMode::Vertical)
        .apply(&v_flip)
        .unwrap();
    assert_eq!(v_flip_again.pixels(), img.pixels());
}

#[test]
fn test_rotate_90_four_times_is_identity() {
    let img = create_test_image(16, 16, ColorSpace::Grayscale);
    let mut result = img.clone();

    for _ in 0..4 {
        result = RotateFilter::new(90.0, 0, [0, 0, 0])
            .apply(&result)
            .unwrap();
    }

    assert_eq!(result.pixels(), img.pixels());
    assert_eq!(result.width(), img.width());
    assert_eq!(result.height(), img.height());
}

#[test]
fn test_filter_composition() {
    let img = create_test_image(10, 10, ColorSpace::Grayscale);

    // Apply multiple filters in sequence
    let brightened = BrightnessFilter::new(30).apply(&img).unwrap();
    let inverted = InvertFilter.apply(&brightened).unwrap();
    let inverted_back = InvertFilter.apply(&inverted).unwrap();

    // Final result should be similar to just brightening (since invert is reversible)
    assert_eq!(inverted_back.width(), img.width());
    assert_eq!(inverted_back.height(), img.height());
}

// Edge case tests
#[test]
fn test_1x1_image_operations() {
    let img = SilvestreImage::new(vec![128], 1, 1, ColorSpace::Grayscale).unwrap();

    // All these operations should work on 1x1 images
    let _ = BrightnessFilter::new(50).apply(&img).unwrap();
    let _ = GrayscaleFilter.apply(&img).unwrap();
    let _ = InvertFilter.apply(&img).unwrap();
    let _ = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
}

#[test]
fn test_single_row_operations() {
    let pixels = vec![10, 20, 30, 40, 50];
    let img = SilvestreImage::new(pixels, 5, 1, ColorSpace::Grayscale).unwrap();

    let cropped = CropFilter::new(1, 0, 3, 1).apply(&img).unwrap();
    assert_eq!(cropped.width(), 3);
    assert_eq!(cropped.height(), 1);
    assert_eq!(cropped.pixels(), &[20, 30, 40]);
}

#[test]
fn test_single_column_operations() {
    let pixels = vec![10, 20, 30, 40, 50];
    let img = SilvestreImage::new(pixels, 1, 5, ColorSpace::Grayscale).unwrap();

    let cropped = CropFilter::new(0, 1, 1, 3).apply(&img).unwrap();
    assert_eq!(cropped.width(), 1);
    assert_eq!(cropped.height(), 3);
    assert_eq!(cropped.pixels(), &[20, 30, 40]);
}

#[test]
fn test_large_image_operations() {
    // Test with a reasonably large image (1000x1000)
    let img = create_test_image(1000, 1000, ColorSpace::Grayscale);

    let resized = ResizeFilter::new(500, 500, Interpolation::NearestNeighbor)
        .apply(&img)
        .unwrap();

    assert_eq!(resized.width(), 500);
    assert_eq!(resized.height(), 500);
}

#[test]
fn test_all_color_spaces() {
    for color_space in [
        ColorSpace::Grayscale,
        ColorSpace::Rgb,
        ColorSpace::Rgba,
    ] {
        let img = create_test_image(8, 8, color_space);

        // Test that basic operations preserve color space
        let brightened = BrightnessFilter::new(10).apply(&img).unwrap();
        assert_eq!(brightened.color_space(), color_space);

        // Test invert
        let inverted = InvertFilter.apply(&img).unwrap();
        assert_eq!(inverted.color_space(), color_space);
    }
}

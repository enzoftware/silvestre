//! Property-based tests using proptest to verify invariants.

use proptest::prelude::*;
use silvestre_core::{
    effects::{GrayscaleFilter, InvertFilter},
    filters::Filter,
    transform::{MirrorFilter, MirrorMode},
    ColorSpace, SilvestreImage,
};

// Strategy for generating valid images
fn arb_image(max_width: u32, max_height: u32) -> impl Strategy<Value = (u32, u32, ColorSpace, Vec<u8>)> {
    (1..=max_width, 1..=max_height, Just(ColorSpace::Grayscale))
        .prop_flat_map(|(w, h, cs)| {
            let len = (w as usize) * (h as usize) * cs.channels();
            (Just(w), Just(h), Just(cs), prop::collection::vec(0u8..=255, len..=len))
        })
}

proptest! {
    #[test]
    fn prop_invert_is_involutory(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels.clone(), width, height, cs).unwrap();
        let inverted = InvertFilter.apply(&img).unwrap();
        let inverted_back = InvertFilter.apply(&inverted).unwrap();

        prop_assert_eq!(inverted_back.pixels(), &pixels);
        prop_assert_eq!(inverted_back.width(), width);
        prop_assert_eq!(inverted_back.height(), height);
    }

    #[test]
    fn prop_horizontal_mirror_is_involutory(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();
        let flipped = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
        let flipped_back = MirrorFilter::new(MirrorMode::Horizontal).apply(&flipped).unwrap();

        prop_assert_eq!(flipped_back.pixels(), img.pixels());
        prop_assert_eq!(flipped_back.width(), img.width());
        prop_assert_eq!(flipped_back.height(), img.height());
    }

    #[test]
    fn prop_vertical_mirror_is_involutory(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();
        let flipped = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
        let flipped_back = MirrorFilter::new(MirrorMode::Vertical).apply(&flipped).unwrap();

        prop_assert_eq!(flipped_back.pixels(), img.pixels());
        prop_assert_eq!(flipped_back.width(), img.width());
        prop_assert_eq!(flipped_back.height(), img.height());
    }

    #[test]
    fn prop_both_mirrors_equal_180_rotation(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();

        let both = MirrorFilter::new(MirrorMode::Both).apply(&img).unwrap();

        let h_then_v = MirrorFilter::new(MirrorMode::Vertical)
            .apply(&MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap())
            .unwrap();

        prop_assert_eq!(both.pixels(), h_then_v.pixels());
    }

    #[test]
    fn prop_grayscale_preserves_dimensions(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();
        let grayed = GrayscaleFilter.apply(&img).unwrap();

        prop_assert_eq!(grayed.width(), width);
        prop_assert_eq!(grayed.height(), height);
        prop_assert_eq!(grayed.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn prop_grayscale_idempotent(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();
        let grayed_once = GrayscaleFilter.apply(&img).unwrap();
        let grayed_twice = GrayscaleFilter.apply(&grayed_once).unwrap();

        // Applying grayscale to an already grayscale image should be identity
        prop_assert_eq!(grayed_twice.pixels(), grayed_once.pixels());
    }

    #[test]
    fn prop_image_dimensions_match_buffer_size(
        (width, height, cs, pixels) in arb_image(100, 100)
    ) {
        let len = (width as usize) * (height as usize) * cs.channels();
        prop_assert_eq!(pixels.len(), len);

        let img = SilvestreImage::new(pixels, width, height, cs).unwrap();
        prop_assert_eq!(img.width(), width);
        prop_assert_eq!(img.height(), height);
    }
}

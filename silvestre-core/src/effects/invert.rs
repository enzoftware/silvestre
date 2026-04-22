//! Colour inversion effect.
//!
//! Inverts each colour channel by computing `255 - value`. The alpha channel
//! (if present) is left unchanged.

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreImage};

/// Colour inversion filter.
///
/// Replaces every colour channel value `v` with `255 - v`. The alpha channel
/// (for [`ColorSpace::Rgba`]) is preserved as-is.
///
/// Applying the filter twice is a round-trip: `invert(invert(img)) == img`.
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::invert::InvertFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![0, 128, 255], 1, 1, ColorSpace::Rgb)?;
/// let out = InvertFilter.apply(&img)?;
/// assert_eq!(out.pixels(), &[255, 127, 0]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InvertFilter;

/// Invert all colour channels of `image`.
///
/// See [`InvertFilter`] for details.
pub fn invert(image: &SilvestreImage) -> Result<SilvestreImage> {
    let cs = image.color_space();
    let channels = cs.channels();
    // For RGBA the alpha channel (index 3 per pixel) is preserved.
    let colour_channels = if cs == ColorSpace::Rgba { 3 } else { channels };

    let mut dst = image.pixels().to_vec();

    for pixel in dst.chunks_exact_mut(channels) {
        for c in 0..colour_channels {
            pixel[c] = 255 - pixel[c];
        }
    }

    SilvestreImage::new(dst, image.width(), image.height(), cs)
}

impl Filter for InvertFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        invert(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(pixels: Vec<u8>, w: u32, h: u32, cs: ColorSpace) -> SilvestreImage {
        SilvestreImage::new(pixels, w, h, cs).unwrap()
    }

    #[test]
    fn black_becomes_white() {
        let image = img(vec![0, 0, 0], 1, 1, ColorSpace::Rgb);
        let out = invert(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 255, 255]);
    }

    #[test]
    fn white_becomes_black() {
        let image = img(vec![255, 255, 255], 1, 1, ColorSpace::Rgb);
        let out = invert(&image).unwrap();
        assert_eq!(out.pixels(), &[0, 0, 0]);
    }

    #[test]
    fn known_channel_values() {
        let image = img(vec![0, 128, 255], 1, 1, ColorSpace::Rgb);
        let out = invert(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 127, 0]);
    }

    #[test]
    fn round_trip_rgb() {
        let pixels = vec![10, 50, 100, 200, 240, 30];
        let image = img(pixels.clone(), 2, 1, ColorSpace::Rgb);
        let out = invert(&invert(&image).unwrap()).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn round_trip_rgba() {
        let pixels = vec![10, 50, 100, 128, 200, 240, 30, 255];
        let image = img(pixels.clone(), 2, 1, ColorSpace::Rgba);
        let out = invert(&invert(&image).unwrap()).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn round_trip_grayscale() {
        let pixels = vec![0, 64, 128, 192, 255];
        let image = img(pixels.clone(), 5, 1, ColorSpace::Grayscale);
        let out = invert(&invert(&image).unwrap()).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn preserves_alpha_channel_rgba() {
        // pixel: [R=100, G=50, B=200, A=77]
        let image = img(vec![100, 50, 200, 77], 1, 1, ColorSpace::Rgba);
        let out = invert(&image).unwrap();
        assert_eq!(out.pixels(), &[155, 205, 55, 77]);
    }

    #[test]
    fn alpha_unchanged_across_multiple_pixels() {
        let image = img(
            vec![0, 0, 0, 100, 255, 255, 255, 200],
            2,
            1,
            ColorSpace::Rgba,
        );
        let out = invert(&image).unwrap();
        // Alpha values must be unchanged.
        assert_eq!(out.pixels()[3], 100);
        assert_eq!(out.pixels()[7], 200);
    }

    #[test]
    fn grayscale_image() {
        let image = img(vec![0, 128, 255], 3, 1, ColorSpace::Grayscale);
        let out = invert(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 127, 0]);
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let image = img(vec![100; 3 * 4 * 3], 4, 3, ColorSpace::Rgb);
        let out = invert(&image).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 3);
        assert_eq!(out.color_space(), ColorSpace::Rgb);
    }

    #[test]
    fn empty_image() {
        let image = img(vec![], 0, 0, ColorSpace::Rgb);
        let out = invert(&image).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn filter_trait_produces_same_result_as_function() {
        let image = img(vec![120, 80, 200], 1, 1, ColorSpace::Rgb);
        let expected = invert(&image).unwrap();
        let out = InvertFilter.apply(&image).unwrap();
        assert_eq!(out.pixels(), expected.pixels());
        assert_eq!(out.color_space(), expected.color_space());
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(InvertFilter);
        let image = img(vec![0, 128, 255], 1, 1, ColorSpace::Rgb);
        let out = filter.apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 127, 0]);
    }

    #[test]
    fn filter_round_trip_via_trait() {
        let image = img(vec![10, 50, 100, 200], 1, 1, ColorSpace::Rgba);
        let inverted = InvertFilter.apply(&image).unwrap();
        let restored = InvertFilter.apply(&inverted).unwrap();
        assert_eq!(restored.pixels(), image.pixels());
    }
}

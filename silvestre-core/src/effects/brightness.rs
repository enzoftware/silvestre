//! Brightness adjustment effect.
//!
//! Adjusts the brightness of an image by adding a constant delta to every
//! colour channel. The alpha channel (if present) is left unchanged. All
//! channel values are clamped to `0..=255`.

use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Brightness adjustment filter.
///
/// Adds `delta` to every colour channel of each pixel, leaving the alpha
/// channel (for [`crate::ColorSpace::Rgba`]) unchanged. Values are clamped to
/// `0..=255`—there is no wraparound.
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::brightness::BrightnessFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100, 150, 200], 1, 1, ColorSpace::Rgb)?;
/// let out = BrightnessFilter::new(50).apply(&img)?;
/// assert_eq!(out.pixels(), &[150, 200, 250]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrightnessFilter {
    delta: i32,
}

impl BrightnessFilter {
    /// Create a new brightness filter with the given `delta`.
    ///
    /// Positive values brighten; negative values darken.
    /// A delta of `0` is a no-op.
    #[must_use]
    pub fn new(delta: i32) -> Self {
        Self { delta }
    }

    /// The delta applied to each colour channel.
    #[must_use]
    pub fn delta(&self) -> i32 {
        self.delta
    }
}

impl Filter for BrightnessFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let cs = image.color_space();
        let src = image.pixels();
        let mut dst = vec![0u8; src.len()];

        if self.delta >= 0 {
            let delta = self.delta.clamp(0, 255) as u8;
            crate::simd::brightness_add(src, &mut dst, delta, cs.channels());
        } else {
            let delta = (-i64::from(self.delta)).clamp(0, 255) as u8;
            crate::simd::brightness_sub(src, &mut dst, delta, cs.channels());
        }

        SilvestreImage::new(dst, image.width(), image.height(), cs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn img(pixels: Vec<u8>, w: u32, h: u32, cs: ColorSpace) -> SilvestreImage {
        SilvestreImage::new(pixels, w, h, cs).unwrap()
    }

    #[test]
    fn zero_delta_is_identity() {
        let image = img(vec![100, 150, 200], 1, 1, ColorSpace::Rgb);
        let out = BrightnessFilter::new(0).apply(&image).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn positive_delta_brightens() {
        let image = img(vec![100, 150, 200], 1, 1, ColorSpace::Rgb);
        let out = BrightnessFilter::new(50).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[150, 200, 250]);
    }

    #[test]
    fn negative_delta_darkens() {
        let image = img(vec![100, 150, 200], 1, 1, ColorSpace::Rgb);
        let out = BrightnessFilter::new(-50).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[50, 100, 150]);
    }

    #[test]
    fn clamps_to_255_on_overflow() {
        let image = img(vec![200, 210, 220], 1, 1, ColorSpace::Rgb);
        let out = BrightnessFilter::new(100).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 255, 255]);
    }

    #[test]
    fn clamps_to_0_on_underflow() {
        let image = img(vec![10, 20, 30], 1, 1, ColorSpace::Rgb);
        let out = BrightnessFilter::new(-100).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[0, 0, 0]);
    }

    #[test]
    fn preserves_alpha_channel_rgba() {
        // pixel: [R=100, G=150, B=200, A=77]  delta=+50
        let image = img(vec![100, 150, 200, 77], 1, 1, ColorSpace::Rgba);
        let out = BrightnessFilter::new(50).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[150, 200, 250, 77]);
    }

    #[test]
    fn alpha_preserved_when_colour_channels_clamp() {
        let image = img(vec![250, 250, 250, 200], 1, 1, ColorSpace::Rgba);
        let out = BrightnessFilter::new(100).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 255, 255, 200]);
    }

    #[test]
    fn grayscale_image() {
        let image = img(vec![100, 200], 2, 1, ColorSpace::Grayscale);
        let out = BrightnessFilter::new(30).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[130, 230]);
    }

    #[test]
    fn empty_image() {
        let image = img(vec![], 0, 0, ColorSpace::Rgb);
        let out = BrightnessFilter::new(50).apply(&image).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let image = img(vec![50; 12], 2, 2, ColorSpace::Rgb);
        let out = BrightnessFilter::new(10).apply(&image).unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
        assert_eq!(out.color_space(), ColorSpace::Rgb);
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(BrightnessFilter::new(10));
        let image = img(vec![100, 100, 100], 1, 1, ColorSpace::Rgb);
        let out = filter.apply(&image).unwrap();
        assert_eq!(out.pixels(), &[110, 110, 110]);
    }

    #[test]
    fn delta_accessor() {
        assert_eq!(BrightnessFilter::new(-30).delta(), -30);
    }

    #[test]
    fn extreme_deltas_do_not_overflow() {
        let image = img(vec![128, 128, 128], 1, 1, ColorSpace::Rgb);
        let min_out = BrightnessFilter::new(i32::MIN).apply(&image).unwrap();
        assert_eq!(min_out.pixels(), &[0, 0, 0]);

        let max_out = BrightnessFilter::new(i32::MAX).apply(&image).unwrap();
        assert_eq!(max_out.pixels(), &[255, 255, 255]);
    }
}

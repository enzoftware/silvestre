//! Brightness adjustment effect.
//!
//! Adjusts the brightness of an image by adding a constant delta to every
//! colour channel. The alpha channel (if present) is left unchanged. All
//! channel values are clamped to `0..=255`.

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreImage};

/// Brightness adjustment filter.
///
/// Adds `delta` to every colour channel of each pixel, leaving the alpha
/// channel (for [`ColorSpace::Rgba`]) unchanged. Values are clamped to
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
        let channels = cs.channels();
        // For RGBA images the alpha channel (index 3 per pixel) is preserved.
        let colour_channels = if cs == ColorSpace::Rgba { 3 } else { channels };

        let src = image.pixels();
        let mut dst = src.to_vec();

        let pixel_count = (image.width() as usize) * (image.height() as usize);
        for i in 0..pixel_count {
            let offset = i * channels;
            for c in 0..colour_channels {
                let v = i32::from(src[offset + c]) + self.delta;
                dst[offset + c] = v.clamp(0, 255) as u8;
            }
        }

        SilvestreImage::new(dst, image.width(), image.height(), cs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

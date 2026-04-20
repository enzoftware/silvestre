//! Contrast adjustment effect.
//!
//! Scales each colour channel relative to the mid-point (128) by a
//! configurable factor. The alpha channel (if present) is left unchanged.
//! All channel values are clamped to `0..=255`.

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreError, SilvestreImage};

/// Contrast adjustment filter.
///
/// Applies `new = 128 + factor * (old - 128)` to every colour channel,
/// leaving the alpha channel (for [`ColorSpace::Rgba`]) unchanged. A
/// `factor` of `1.0` is a no-op; values above `1.0` increase contrast;
/// values between `0.0` and `1.0` reduce contrast; `0.0` flattens
/// everything to the midpoint. `factor` must be non-negative and finite.
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::contrast::ContrastFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // factor 1.0 is a no-op
/// let img = SilvestreImage::new(vec![80, 128, 200], 1, 1, ColorSpace::Rgb)?;
/// let out = ContrastFilter::new(1.0)?.apply(&img)?;
/// assert_eq!(out.pixels(), &[80, 128, 200]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContrastFilter {
    factor: f32,
}

impl ContrastFilter {
    /// Create a new contrast filter with the given `factor`.
    ///
    /// Returns an error if `factor` is negative or not finite.
    pub fn new(factor: f32) -> Result<Self> {
        if !factor.is_finite() || factor < 0.0 {
            return Err(SilvestreError::InvalidParameter(
                "contrast factor must be a non-negative finite number".into(),
            ));
        }
        Ok(Self { factor })
    }

    /// The scale factor applied around the mid-point 128.
    #[must_use]
    pub fn factor(&self) -> f32 {
        self.factor
    }
}

impl Filter for ContrastFilter {
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
                let old = f32::from(src[offset + c]);
                let new = 128.0 + self.factor * (old - 128.0);
                dst[offset + c] = new.round().clamp(0.0, 255.0) as u8;
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
    fn factor_one_is_identity() {
        let image = img(vec![80, 128, 200], 1, 1, ColorSpace::Rgb);
        let out = ContrastFilter::new(1.0).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn midpoint_unchanged_by_any_factor() {
        // 128 is the fixed point: 128 + factor*(128-128) = 128
        let image = img(vec![128, 128, 128], 1, 1, ColorSpace::Rgb);
        let out = ContrastFilter::new(2.5).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[128, 128, 128]);
    }

    #[test]
    fn factor_above_one_increases_contrast() {
        // pixel=200, factor=2.0: 128 + 2*(200-128) = 272 -> 255
        let image = img(vec![200], 1, 1, ColorSpace::Grayscale);
        let out = ContrastFilter::new(2.0).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255]);
    }

    #[test]
    fn factor_below_one_reduces_contrast() {
        // pixel=200, factor=0.5: 128 + 0.5*(200-128) = 164
        let image = img(vec![200], 1, 1, ColorSpace::Grayscale);
        let out = ContrastFilter::new(0.5).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[164]);
    }

    #[test]
    fn factor_zero_flattens_to_midpoint() {
        // Any pixel maps to 128 + 0*(v-128) = 128
        let image = img(vec![0, 50, 128, 200, 255], 5, 1, ColorSpace::Grayscale);
        let out = ContrastFilter::new(0.0).unwrap().apply(&image).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 128));
    }

    #[test]
    fn clamps_lower_end_to_zero() {
        // pixel=10, factor=3.0: 128 + 3*(10-128) = -226 -> 0
        let image = img(vec![10], 1, 1, ColorSpace::Grayscale);
        let out = ContrastFilter::new(3.0).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[0]);
    }

    #[test]
    fn clamps_upper_end_to_255() {
        // pixel=240, factor=3.0: 128 + 3*(240-128) = 464 -> 255
        let image = img(vec![240], 1, 1, ColorSpace::Grayscale);
        let out = ContrastFilter::new(3.0).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255]);
    }

    #[test]
    fn preserves_alpha_channel_rgba() {
        // R=200, G=128, B=50, A=99  factor=2.0
        // R: 128 + 2*(200-128) = 272 -> 255
        // G: 128 + 2*(128-128) = 128
        // B: 128 + 2*(50-128)  = -28 -> 0
        // A: 99 (unchanged)
        let image = img(vec![200, 128, 50, 99], 1, 1, ColorSpace::Rgba);
        let out = ContrastFilter::new(2.0).unwrap().apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 128, 0, 99]);
    }

    #[test]
    fn rejects_negative_factor() {
        assert!(matches!(
            ContrastFilter::new(-1.0),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn rejects_nan_factor() {
        assert!(matches!(
            ContrastFilter::new(f32::NAN),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn rejects_infinite_factor() {
        assert!(matches!(
            ContrastFilter::new(f32::INFINITY),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn empty_image() {
        let image = img(vec![], 0, 0, ColorSpace::Rgb);
        let out = ContrastFilter::new(2.0).unwrap().apply(&image).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let image = img(vec![100; 9], 3, 3, ColorSpace::Grayscale);
        let out = ContrastFilter::new(1.5).unwrap().apply(&image).unwrap();
        assert_eq!(out.width(), 3);
        assert_eq!(out.height(), 3);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(ContrastFilter::new(1.0).unwrap());
        let image = img(vec![100, 128, 200], 1, 1, ColorSpace::Rgb);
        let out = filter.apply(&image).unwrap();
        assert_eq!(out.pixels(), image.pixels());
    }

    #[test]
    fn factor_accessor() {
        let f = ContrastFilter::new(1.5).unwrap();
        assert!((f.factor() - 1.5).abs() < f32::EPSILON);
    }
}

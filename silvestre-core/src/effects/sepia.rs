//! Sepia tone effect.
//!
//! Applies the standard sepia matrix transform to each RGB pixel, producing a
//! warm brownish tone. The alpha channel (if present) is left unchanged. All
//! channel values are clamped to `0..=255`.

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreImage};

/// Sepia tone filter.
///
/// Applies the standard sepia colour-matrix to every pixel:
///
/// ```text
/// R_out = 0.393·R + 0.769·G + 0.189·B
/// G_out = 0.349·R + 0.686·G + 0.168·B
/// B_out = 0.272·R + 0.534·G + 0.131·B
/// ```
///
/// The alpha channel (for [`ColorSpace::Rgba`]) is left unchanged.
/// Grayscale images are treated as if each grey value fills all three colour
/// channels before the matrix is applied (the output is still grayscale).
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::sepia::SepiaFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // Mid-grey through the sepia matrix → warm brownish tone.
/// let img = SilvestreImage::new(vec![100, 100, 100], 1, 1, ColorSpace::Rgb)?;
/// let out = SepiaFilter.apply(&img)?;
/// assert_eq!(out.pixels(), &[135, 120, 94]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SepiaFilter;

/// Apply sepia tone to `image`.
///
/// See [`SepiaFilter`] for the exact formula.
pub fn to_sepia(image: &SilvestreImage) -> Result<SilvestreImage> {
    let cs = image.color_space();
    let channels = cs.channels();
    let src = image.pixels();
    let pixel_count = (image.width() as usize) * (image.height() as usize);
    let mut dst = src.to_vec();

    for i in 0..pixel_count {
        let offset = i * channels;

        let (r, g, b) = match cs {
            ColorSpace::Grayscale => {
                let v = f32::from(src[offset]);
                (v, v, v)
            }
            ColorSpace::Rgb | ColorSpace::Rgba => {
                let r = f32::from(src[offset]);
                let g = f32::from(src[offset + 1]);
                let b = f32::from(src[offset + 2]);
                (r, g, b)
            }
        };

        let r_out = (0.393 * r + 0.769 * g + 0.189 * b).round().clamp(0.0, 255.0) as u8;
        let g_out = (0.349 * r + 0.686 * g + 0.168 * b).round().clamp(0.0, 255.0) as u8;
        let b_out = (0.272 * r + 0.534 * g + 0.131 * b).round().clamp(0.0, 255.0) as u8;

        match cs {
            ColorSpace::Grayscale => {
                // Collapse back to luminance so the output stays grayscale.
                let lum = (0.299 * f32::from(r_out)
                    + 0.587 * f32::from(g_out)
                    + 0.114 * f32::from(b_out))
                .round()
                .clamp(0.0, 255.0) as u8;
                dst[offset] = lum;
            }
            ColorSpace::Rgb => {
                dst[offset] = r_out;
                dst[offset + 1] = g_out;
                dst[offset + 2] = b_out;
            }
            ColorSpace::Rgba => {
                dst[offset] = r_out;
                dst[offset + 1] = g_out;
                dst[offset + 2] = b_out;
                // dst[offset + 3] (alpha) left unchanged from src.to_vec()
            }
        }
    }

    SilvestreImage::new(dst, image.width(), image.height(), cs)
}

impl Filter for SepiaFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        to_sepia(image)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(pixels: Vec<u8>, w: u32, h: u32, cs: ColorSpace) -> SilvestreImage {
        SilvestreImage::new(pixels, w, h, cs).unwrap()
    }

    // Helper: apply the sepia matrix to (r,g,b) and return (r_out, g_out, b_out).
    fn sepia_pixel(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        let (rf, gf, bf) = (f32::from(r), f32::from(g), f32::from(b));
        let r_out = (0.393 * rf + 0.769 * gf + 0.189 * bf).round().clamp(0.0, 255.0) as u8;
        let g_out = (0.349 * rf + 0.686 * gf + 0.168 * bf).round().clamp(0.0, 255.0) as u8;
        let b_out = (0.272 * rf + 0.534 * gf + 0.131 * bf).round().clamp(0.0, 255.0) as u8;
        (r_out, g_out, b_out)
    }

    #[test]
    fn black_stays_black() {
        let image = img(vec![0, 0, 0], 1, 1, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        assert_eq!(out.pixels(), &[0, 0, 0]);
    }

    #[test]
    fn white_produces_warm_tone() {
        // White: R and G clamp to 255; B = round(0.937*255) = 239.
        let image = img(vec![255, 255, 255], 1, 1, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        let (r, g, b) = sepia_pixel(255, 255, 255);
        assert_eq!(out.pixels(), &[r, g, b]);
        // Channels are ordered r >= g >= b (warm tone).
        assert!(out.pixels()[0] >= out.pixels()[1]);
        assert!(out.pixels()[1] >= out.pixels()[2]);
    }

    #[test]
    fn mid_grey_produces_warm_brownish_tone() {
        // 100,100,100 → R=135, G=120, B=94  (no clamping, visible warm shift)
        let image = img(vec![100, 100, 100], 1, 1, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        let (r, g, b) = sepia_pixel(100, 100, 100);
        assert_eq!(out.pixels(), &[r, g, b]);
        assert!(r > g && g > b);
    }

    #[test]
    fn pure_red_rgb() {
        let image = img(vec![255, 0, 0], 1, 1, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        let (r, g, b) = sepia_pixel(255, 0, 0);
        assert_eq!(out.pixels(), &[r, g, b]);
    }

    #[test]
    fn preserves_alpha_channel_rgba() {
        // pixel: [R=100, G=150, B=200, A=77]
        let image = img(vec![100, 150, 200, 77], 1, 1, ColorSpace::Rgba);
        let out = to_sepia(&image).unwrap();
        let (r, g, b) = sepia_pixel(100, 150, 200);
        assert_eq!(out.pixels(), &[r, g, b, 77]);
    }

    #[test]
    fn alpha_unchanged_when_channels_clamp() {
        let image = img(vec![255, 255, 255, 200], 1, 1, ColorSpace::Rgba);
        let out = to_sepia(&image).unwrap();
        assert_eq!(out.pixels()[3], 200);
    }

    #[test]
    fn grayscale_image_stays_grayscale() {
        let image = img(vec![128], 1, 1, ColorSpace::Grayscale);
        let out = to_sepia(&image).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        assert_eq!(out.pixels().len(), 1);
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let image = img(vec![100; 3 * 4 * 3], 4, 3, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 3);
        assert_eq!(out.color_space(), ColorSpace::Rgb);
    }

    #[test]
    fn empty_image() {
        let image = img(vec![], 0, 0, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn multi_pixel_known_values() {
        // Two pixels: pure red and pure green.
        let image = img(vec![255, 0, 0, 0, 255, 0], 2, 1, ColorSpace::Rgb);
        let out = to_sepia(&image).unwrap();
        let (r1, g1, b1) = sepia_pixel(255, 0, 0);
        let (r2, g2, b2) = sepia_pixel(0, 255, 0);
        assert_eq!(out.pixels(), &[r1, g1, b1, r2, g2, b2]);
    }

    #[test]
    fn filter_trait_produces_same_result_as_function() {
        let image = img(vec![120, 80, 200], 1, 1, ColorSpace::Rgb);
        let expected = to_sepia(&image).unwrap();
        let out = SepiaFilter.apply(&image).unwrap();
        assert_eq!(out.pixels(), expected.pixels());
        assert_eq!(out.color_space(), expected.color_space());
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(SepiaFilter);
        let image = img(vec![255, 255, 255], 1, 1, ColorSpace::Rgb);
        let out = filter.apply(&image).unwrap();
        let (r, g, b) = sepia_pixel(255, 255, 255);
        assert_eq!(out.pixels(), &[r, g, b]);
    }
}

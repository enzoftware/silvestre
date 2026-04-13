//! Grayscale conversion effect.
//!
//! Converts an image to single-channel grayscale using the ITU-R BT.601
//! luminance formula: `Y = 0.299·R + 0.587·G + 0.114·B`.

use crate::{ColorSpace, Result, SilvestreImage};

/// Convert an image to grayscale.
///
/// If the image is already [`ColorSpace::Grayscale`], a clone is returned.
/// For [`ColorSpace::Rgb`] and [`ColorSpace::Rgba`], the standard luminance
/// weights are applied and the alpha channel (if present) is discarded.
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::grayscale::to_grayscale;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let rgb = SilvestreImage::new(vec![255, 0, 0, 0, 255, 0], 2, 1, ColorSpace::Rgb)?;
/// let gray = to_grayscale(&rgb)?;
/// assert_eq!(gray.color_space(), ColorSpace::Grayscale);
/// assert_eq!(gray.width(), 2);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
pub fn to_grayscale(image: &SilvestreImage) -> Result<SilvestreImage> {
    let width = image.width();
    let height = image.height();

    match image.color_space() {
        ColorSpace::Grayscale => {
            return SilvestreImage::new(
                image.pixels().to_vec(),
                width,
                height,
                ColorSpace::Grayscale,
            );
        }
        ColorSpace::Rgb | ColorSpace::Rgba => {}
    }

    let src = image.pixels();
    let channels = image.color_space().channels();
    let pixel_count = (width as usize) * (height as usize);
    let mut dst = Vec::with_capacity(pixel_count);

    for i in 0..pixel_count {
        let offset = i * channels;
        let r = f32::from(src[offset]);
        let g = f32::from(src[offset + 1]);
        let b = f32::from(src[offset + 2]);
        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
        dst.push(lum.round().clamp(0.0, 255.0) as u8);
    }

    SilvestreImage::new(dst, width, height, ColorSpace::Grayscale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grayscale_passthrough() {
        let img = SilvestreImage::new(vec![42; 9], 3, 3, ColorSpace::Grayscale).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn rgb_to_grayscale() {
        // Pure red: 0.299 * 255 = 76.245 -> 76
        let img = SilvestreImage::new(vec![255, 0, 0], 1, 1, ColorSpace::Rgb).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        assert_eq!(out.pixels(), &[76]);
    }

    #[test]
    fn rgba_to_grayscale_ignores_alpha() {
        // Pure green with varying alpha: 0.587 * 255 = 149.685 -> 150
        let img =
            SilvestreImage::new(vec![0, 255, 0, 128, 0, 255, 0, 0], 2, 1, ColorSpace::Rgba)
                .unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.pixels(), &[150, 150]);
    }

    #[test]
    fn white_pixel() {
        // White: 0.299*255 + 0.587*255 + 0.114*255 = 255
        let img = SilvestreImage::new(vec![255, 255, 255], 1, 1, ColorSpace::Rgb).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.pixels(), &[255]);
    }

    #[test]
    fn black_pixel() {
        let img = SilvestreImage::new(vec![0, 0, 0], 1, 1, ColorSpace::Rgb).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.pixels(), &[0]);
    }

    #[test]
    fn empty_image() {
        let img = SilvestreImage::new(vec![], 0, 0, ColorSpace::Rgb).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.width(), 0);
        assert_eq!(out.height(), 0);
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn preserves_dimensions() {
        let img = SilvestreImage::new(vec![100; 3 * 4 * 5], 4, 5, ColorSpace::Rgb).unwrap();
        let out = to_grayscale(&img).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 5);
        assert_eq!(out.pixels().len(), 20);
    }
}

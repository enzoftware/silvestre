//! Sobel edge-detection filter.
//!
//! Computes the gradient magnitude of a grayscale image using the classic
//! 3x3 Sobel operator. The public [`SobelFilter`] implements the [`Filter`]
//! trait and returns a `u8` magnitude image. Internal helpers expose raw
//! `f32` gradient data for use by the Canny pipeline.

use crate::filters::convolution::{resolve_coord, BorderMode};
use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreError, SilvestreImage};

/// Sobel edge-detection filter.
///
/// Applies the 3x3 Sobel kernels in the X and Y directions and returns
/// the gradient magnitude image (clamped to `0..=255`). The input should
/// be a grayscale image; for color images the filter returns an error.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{Filter, SobelFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale)?;
/// let sobel = SobelFilter::new();
/// let edges = sobel.apply(&img)?;
/// assert_eq!(edges.color_space(), ColorSpace::Grayscale);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SobelFilter {
    border: BorderMode,
}

impl SobelFilter {
    /// Create a new Sobel filter with [`BorderMode::Clamp`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            border: BorderMode::Clamp,
        }
    }

    /// Create a new Sobel filter with an explicit border mode.
    #[must_use]
    pub fn with_border(border: BorderMode) -> Self {
        Self { border }
    }
}

impl Default for SobelFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for SobelFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        if image.color_space() != ColorSpace::Grayscale {
            return Err(SilvestreError::UnsupportedColorSpace(image.color_space()));
        }

        let width = image.width();
        let height = image.height();

        if width == 0 || height == 0 {
            return SilvestreImage::new(vec![], width, height, ColorSpace::Grayscale);
        }

        let (gx, gy) = sobel_gradients(image, self.border);
        let len = gx.len();
        let mut dst = Vec::with_capacity(len);
        for i in 0..len {
            let mag = (gx[i] * gx[i] + gy[i] * gy[i]).sqrt();
            dst.push(mag.round().clamp(0.0, 255.0) as u8);
        }
        SilvestreImage::new(dst, width, height, ColorSpace::Grayscale)
    }
}

/// Compute the raw Sobel gradients (Gx, Gy) as `f32` buffers.
///
/// Each buffer has `width * height` elements in row-major order. The input
/// **must** be a grayscale image with positive dimensions; the caller is
/// responsible for validating this.
pub(crate) fn sobel_gradients(
    image: &SilvestreImage,
    border: BorderMode,
) -> (Vec<f32>, Vec<f32>) {
    debug_assert_eq!(image.color_space(), ColorSpace::Grayscale);
    let width = image.width();
    let height = image.height();
    let src = image.pixels();
    let w = width as usize;
    let len = w * (height as usize);
    let mut gx = vec![0.0_f32; len];
    let mut gy = vec![0.0_f32; len];

    // Sobel kernels:
    //   Gx = [-1  0  1]    Gy = [-1 -2 -1]
    //        [-2  0  2]         [ 0  0  0]
    //        [-1  0  1]         [ 1  2  1]
    #[rustfmt::skip]
    let kx: [(i64, i64, f32); 6] = [
        (-1, -1, -1.0), (1, -1,  1.0),
        (-1,  0, -2.0), (1,  0,  2.0),
        (-1,  1, -1.0), (1,  1,  1.0),
    ];
    #[rustfmt::skip]
    let ky: [(i64, i64, f32); 6] = [
        (-1, -1, -1.0), (0, -1, -2.0), (1, -1, -1.0),
        (-1,  1,  1.0), (0,  1,  2.0), (1,  1,  1.0),
    ];

    for y in 0..height as i64 {
        for x in 0..width as i64 {
            let mut sx = 0.0_f32;
            let mut sy = 0.0_f32;
            for &(dx, dy, weight) in &kx {
                let val = sample(src, x + dx, y + dy, width, height, border);
                sx += weight * val;
            }
            for &(dx, dy, weight) in &ky {
                let val = sample(src, x + dx, y + dy, width, height, border);
                sy += weight * val;
            }
            let idx = (y as usize) * (width as usize) + (x as usize);
            gx[idx] = sx;
            gy[idx] = sy;
        }
    }

    (gx, gy)
}

/// Sample a single grayscale pixel, resolving out-of-bounds via `border`.
fn sample(src: &[u8], x: i64, y: i64, width: u32, height: u32, border: BorderMode) -> f32 {
    match resolve_coord(x, y, width, height, border) {
        Some((sx, sy)) => f32::from(src[sy * (width as usize) + sx]),
        None => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn rejects_rgb_image() {
        let img = SilvestreImage::new(vec![0; 9], 3, 1, ColorSpace::Rgb).unwrap();
        let err = SobelFilter::new().apply(&img).unwrap_err();
        assert!(matches!(err, SilvestreError::UnsupportedColorSpace(_)));
    }

    #[test]
    fn rejects_rgba_image() {
        let img = SilvestreImage::new(vec![0; 4], 1, 1, ColorSpace::Rgba).unwrap();
        let err = SobelFilter::new().apply(&img).unwrap_err();
        assert!(matches!(err, SilvestreError::UnsupportedColorSpace(_)));
    }

    #[test]
    fn uniform_image_yields_zero_edges() {
        let img = gray_image(5, 5, vec![128; 25]);
        let out = SobelFilter::new().apply(&img).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 0));
    }

    #[test]
    fn vertical_edge_detected() {
        // Left half black, right half white -> strong vertical edge.
        #[rustfmt::skip]
        let pixels = vec![
            0, 0, 255, 255,
            0, 0, 255, 255,
            0, 0, 255, 255,
            0, 0, 255, 255,
        ];
        let img = gray_image(4, 4, pixels);
        let out = SobelFilter::new().apply(&img).unwrap();
        // The edge pixels (columns 1 and 2) should have high magnitude.
        // Column 0 and 3 may also respond due to clamped border.
        let row1 = &out.pixels()[4..8];
        assert!(row1[1] > 100, "edge at col 1 should be strong: {}", row1[1]);
        assert!(row1[2] > 100, "edge at col 2 should be strong: {}", row1[2]);
    }

    #[test]
    fn horizontal_edge_detected() {
        // Top half black, bottom half white -> strong horizontal edge.
        #[rustfmt::skip]
        let pixels = vec![
            0,   0,   0,   0,
            0,   0,   0,   0,
            255, 255, 255, 255,
            255, 255, 255, 255,
        ];
        let img = gray_image(4, 4, pixels);
        let out = SobelFilter::new().apply(&img).unwrap();
        // Rows 1 and 2 should have strong responses.
        let row1 = &out.pixels()[4..8];
        let row2 = &out.pixels()[8..12];
        assert!(row1[1] > 100, "edge at (1,1) should be strong");
        assert!(row2[1] > 100, "edge at (1,2) should be strong");
    }

    #[test]
    fn preserves_dimensions() {
        let img = gray_image(6, 4, vec![50; 24]);
        let out = SobelFilter::new().apply(&img).unwrap();
        assert_eq!(out.width(), 6);
        assert_eq!(out.height(), 4);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn empty_image() {
        let img = gray_image(0, 0, vec![]);
        let out = SobelFilter::new().apply(&img).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn single_pixel() {
        let img = gray_image(1, 1, vec![128]);
        let out = SobelFilter::new().apply(&img).unwrap();
        // With clamping, all neighbors are the same -> zero gradient.
        assert_eq!(out.pixels(), &[0]);
    }

    #[test]
    fn default_trait() {
        let filter = SobelFilter::default();
        let img = gray_image(3, 3, vec![50; 9]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 3);
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(SobelFilter::new());
        let img = gray_image(3, 3, vec![50; 9]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 3);
    }

    #[test]
    fn with_border_zero_differs_from_clamp() {
        // With Zero border, edge pixels see 0 outside, creating gradients.
        // With Clamp, all neighbors are 255, so gradients are 0.
        let img = gray_image(3, 3, vec![255; 9]);
        let clamp_out = SobelFilter::new().apply(&img).unwrap();
        let zero_out = SobelFilter::with_border(BorderMode::Zero).apply(&img).unwrap();
        assert_ne!(clamp_out.pixels(), zero_out.pixels());
    }

    #[test]
    fn sobel_gradients_direction() {
        // A simple image with a vertical edge: left=0, right=255
        #[rustfmt::skip]
        let pixels = vec![
            0, 255,
            0, 255,
            0, 255,
        ];
        let img = gray_image(2, 3, pixels);
        let (gx, gy) = sobel_gradients(&img, BorderMode::Clamp);
        // All Gx values should be positive (left-to-right intensity increase).
        // Gy at interior rows should be ~0 (uniform vertically).
        assert!(gx.iter().all(|&v| v >= 0.0), "gx should be non-negative");
        // Middle row gy should be near zero.
        assert!(
            gy[2].abs() < 1e-6 && gy[3].abs() < 1e-6,
            "gy at middle row should be ~0"
        );
    }
}

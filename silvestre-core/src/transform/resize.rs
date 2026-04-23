//! Image resize transform.
//!
//! Scales an image to arbitrary dimensions using either nearest-neighbor
//! (fast, blocky) or bilinear (smooth) interpolation. The color space and
//! channel layout are preserved; the pixel buffer is re-sampled into a new
//! allocation of exactly `target_width × target_height × channels` bytes.

use crate::filters::Filter;
use crate::{Result, SilvestreError, SilvestreImage};

/// Interpolation algorithm used when resampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interpolation {
    /// Map each destination pixel to the nearest source pixel.
    ///
    /// Fast and produces a pixelated / blocky result. Suitable for pixel-art
    /// or when speed is preferred over quality.
    NearestNeighbor,
    /// Blend the four surrounding source pixels using area-weighted averages.
    ///
    /// Slower than nearest-neighbor but produces smoother results, especially
    /// when upscaling.
    Bilinear,
}

/// Resize filter.
///
/// Scales `image` to `target_width × target_height` using the chosen
/// [`Interpolation`] method. The source color space is preserved unchanged.
///
/// If either target dimension is zero an empty image (0 × 0) is returned.
///
/// # Examples
///
/// ```
/// use silvestre_core::transform::resize::{Interpolation, ResizeFilter};
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // 2×2 grayscale image upscaled to 4×4 with nearest-neighbor.
/// let src = SilvestreImage::new(
///     vec![10, 20, 30, 40],
///     2, 2,
///     ColorSpace::Grayscale,
/// )?;
/// let out = ResizeFilter::new(4, 4, Interpolation::NearestNeighbor).apply(&src)?;
/// assert_eq!(out.width(), 4);
/// assert_eq!(out.height(), 4);
/// // Top-left quadrant maps to the source pixel (0,0) = 10.
/// assert_eq!(out.get_pixel(0, 0)?, &[10]);
/// assert_eq!(out.get_pixel(1, 0)?, &[10]);
/// // Top-right quadrant maps to source pixel (1,0) = 20.
/// assert_eq!(out.get_pixel(2, 0)?, &[20]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResizeFilter {
    target_width: u32,
    target_height: u32,
    interpolation: Interpolation,
}

impl ResizeFilter {
    /// Create a new resize filter.
    ///
    /// `target_width` and `target_height` are the desired output dimensions.
    /// Passing zero for either produces an empty output image.
    #[must_use]
    pub fn new(target_width: u32, target_height: u32, interpolation: Interpolation) -> Self {
        Self {
            target_width,
            target_height,
            interpolation,
        }
    }

    /// Target output width in pixels.
    #[must_use]
    pub fn target_width(&self) -> u32 {
        self.target_width
    }

    /// Target output height in pixels.
    #[must_use]
    pub fn target_height(&self) -> u32 {
        self.target_height
    }

    /// Interpolation method used when resampling.
    #[must_use]
    pub fn interpolation(&self) -> Interpolation {
        self.interpolation
    }
}

impl Filter for ResizeFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let dst_w = self.target_width as usize;
        let dst_h = self.target_height as usize;

        if dst_w == 0 || dst_h == 0 {
            return SilvestreImage::new(vec![], 0, 0, image.color_space());
        }

        let src_w = image.width() as usize;
        let src_h = image.height() as usize;

        if src_w == 0 || src_h == 0 {
            return SilvestreImage::new(vec![], 0, 0, image.color_space());
        }

        let channels = image.color_space().channels();
        let src = image.pixels();
        let buf_size = dst_w
            .checked_mul(dst_h)
            .and_then(|p| p.checked_mul(channels))
            .ok_or(SilvestreError::InvalidDimensions {
                width: self.target_width,
                height: self.target_height,
            })?;
        let mut dst = Vec::<u8>::new();
        dst.try_reserve_exact(buf_size)
            .map_err(|_| SilvestreError::InvalidDimensions {
                width: self.target_width,
                height: self.target_height,
            })?;
        dst.resize(buf_size, 0);

        match self.interpolation {
            Interpolation::NearestNeighbor => {
                for dst_y in 0..dst_h {
                    // Map destination row to the nearest source row (floor).
                    let src_y = ((dst_y as u64 * src_h as u64) / dst_h as u64) as usize;
                    for dst_x in 0..dst_w {
                        let src_x = ((dst_x as u64 * src_w as u64) / dst_w as u64) as usize;
                        let src_off = (src_y * src_w + src_x) * channels;
                        let dst_off = (dst_y * dst_w + dst_x) * channels;
                        dst[dst_off..dst_off + channels]
                            .copy_from_slice(&src[src_off..src_off + channels]);
                    }
                }
            }
            Interpolation::Bilinear => {
                for dst_y in 0..dst_h {
                    // Map destination pixel center to continuous source coordinates.
                    let sy = (dst_y as f32 + 0.5) * src_h as f32 / dst_h as f32 - 0.5;
                    let sy = sy.max(0.0);
                    let y0 = (sy.floor() as usize).min(src_h - 1);
                    let y1 = (y0 + 1).min(src_h - 1);
                    let ty = sy - sy.floor();

                    for dst_x in 0..dst_w {
                        let sx = (dst_x as f32 + 0.5) * src_w as f32 / dst_w as f32 - 0.5;
                        let sx = sx.max(0.0);
                        let x0 = (sx.floor() as usize).min(src_w - 1);
                        let x1 = (x0 + 1).min(src_w - 1);
                        let tx = sx - sx.floor();

                        let dst_off = (dst_y * dst_w + dst_x) * channels;

                        // Offsets for the four surrounding source pixels.
                        let off00 = (y0 * src_w + x0) * channels;
                        let off10 = (y0 * src_w + x1) * channels;
                        let off01 = (y1 * src_w + x0) * channels;
                        let off11 = (y1 * src_w + x1) * channels;

                        for c in 0..channels {
                            let p00 = src[off00 + c] as f32;
                            let p10 = src[off10 + c] as f32;
                            let p01 = src[off01 + c] as f32;
                            let p11 = src[off11 + c] as f32;

                            let top = p00 * (1.0 - tx) + p10 * tx;
                            let bot = p01 * (1.0 - tx) + p11 * tx;
                            let val = top * (1.0 - ty) + bot * ty;
                            dst[dst_off + c] = val.round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }

        SilvestreImage::new(dst, self.target_width, self.target_height, image.color_space())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSpace, SilvestreImage};

    fn gray(w: u32, h: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, w, h, ColorSpace::Grayscale).unwrap()
    }

    // ------------------------------------------------------------------ //
    // Identity (same dimensions)
    // ------------------------------------------------------------------ //

    #[test]
    fn nearest_same_dimensions_is_identity() {
        let img = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let out = ResizeFilter::new(3, 2, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.pixels(), img.pixels());
        assert_eq!(out.width(), 3);
        assert_eq!(out.height(), 2);
    }

    #[test]
    fn bilinear_same_dimensions_is_identity() {
        let img = gray(3, 2, vec![10, 20, 30, 40, 50, 60]);
        let out = ResizeFilter::new(3, 2, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    // ------------------------------------------------------------------ //
    // Nearest-neighbor upscale 2×2 → 4×4 (from issue acceptance criteria)
    // ------------------------------------------------------------------ //

    #[test]
    fn nearest_upscale_2x2_to_4x4_pixel_values() {
        // Source:
        //   10 20
        //   30 40
        let img = gray(2, 2, vec![10, 20, 30, 40]);
        let out = ResizeFilter::new(4, 4, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 4);

        // Each source pixel maps to a 2×2 block in the destination.
        // Top-left block → 10
        assert_eq!(out.get_pixel(0, 0).unwrap(), &[10]);
        assert_eq!(out.get_pixel(1, 0).unwrap(), &[10]);
        assert_eq!(out.get_pixel(0, 1).unwrap(), &[10]);
        assert_eq!(out.get_pixel(1, 1).unwrap(), &[10]);
        // Top-right block → 20
        assert_eq!(out.get_pixel(2, 0).unwrap(), &[20]);
        assert_eq!(out.get_pixel(3, 0).unwrap(), &[20]);
        assert_eq!(out.get_pixel(2, 1).unwrap(), &[20]);
        assert_eq!(out.get_pixel(3, 1).unwrap(), &[20]);
        // Bottom-left block → 30
        assert_eq!(out.get_pixel(0, 2).unwrap(), &[30]);
        assert_eq!(out.get_pixel(1, 2).unwrap(), &[30]);
        assert_eq!(out.get_pixel(0, 3).unwrap(), &[30]);
        assert_eq!(out.get_pixel(1, 3).unwrap(), &[30]);
        // Bottom-right block → 40
        assert_eq!(out.get_pixel(2, 2).unwrap(), &[40]);
        assert_eq!(out.get_pixel(3, 2).unwrap(), &[40]);
        assert_eq!(out.get_pixel(2, 3).unwrap(), &[40]);
        assert_eq!(out.get_pixel(3, 3).unwrap(), &[40]);
    }

    // ------------------------------------------------------------------ //
    // Bilinear upscale 2×2 → 4×4 (from issue acceptance criteria)
    // ------------------------------------------------------------------ //

    #[test]
    fn bilinear_upscale_2x2_to_4x4_pixel_values() {
        // Source:
        //   0   255
        //   255 0
        let img = gray(2, 2, vec![0, 255, 255, 0]);
        let out = ResizeFilter::new(4, 4, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 4);

        // Corners of the destination map exactly to source corners.
        assert_eq!(out.get_pixel(0, 0).unwrap(), &[0]);
        assert_eq!(out.get_pixel(3, 0).unwrap(), &[255]);
        assert_eq!(out.get_pixel(0, 3).unwrap(), &[255]);
        assert_eq!(out.get_pixel(3, 3).unwrap(), &[0]);

        // The bilinear output must be strictly between the min and max of the
        // source, showing that blending occurred.
        let center_tl = out.get_pixel(1, 1).unwrap()[0];
        let center_tr = out.get_pixel(2, 1).unwrap()[0];
        assert!(center_tl < 255 && center_tl > 0, "expected blending at (1,1), got {center_tl}");
        assert!(center_tr < 255 && center_tr > 0, "expected blending at (2,1), got {center_tr}");
    }

    #[test]
    fn bilinear_smoother_than_nearest_for_upscale() {
        // Scale a two-pixel gradient [0, 255] to 10 pixels.
        // Nearest-neighbor produces a single large jump (e.g. 255 in one step).
        // Bilinear spreads the transition across many small steps.
        let img = gray(2, 1, vec![0, 255]);
        let nn = ResizeFilter::new(10, 1, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        let bl = ResizeFilter::new(10, 1, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();

        // Maximum single-step change: bilinear must be smaller.
        let max_nn_step = nn
            .pixels()
            .windows(2)
            .map(|w| w[1].abs_diff(w[0]))
            .max()
            .unwrap_or(0);
        let max_bl_step = bl
            .pixels()
            .windows(2)
            .map(|w| w[1].abs_diff(w[0]))
            .max()
            .unwrap_or(0);

        assert!(
            max_bl_step < max_nn_step,
            "bilinear max step ({max_bl_step}) should be smaller than nearest-neighbor ({max_nn_step})"
        );
    }

    // ------------------------------------------------------------------ //
    // Downscaling
    // ------------------------------------------------------------------ //

    #[test]
    fn nearest_downscale_4x1_to_2x1() {
        // 4 pixels → 2: each output pixel takes the first of its two source pixels.
        let img = gray(4, 1, vec![10, 20, 30, 40]);
        let out = ResizeFilter::new(2, 1, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 1);
        assert_eq!(out.pixels(), &[10, 30]);
    }

    #[test]
    fn bilinear_downscale_produces_correct_dimensions() {
        let img = gray(4, 4, vec![100; 16]);
        let out = ResizeFilter::new(2, 2, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
        // Uniform source → uniform output.
        assert!(out.pixels().iter().all(|&v| v == 100));
    }

    // ------------------------------------------------------------------ //
    // Multi-channel images
    // ------------------------------------------------------------------ //

    #[test]
    fn nearest_upscale_rgb() {
        // 1×1 RGB image → 2×2
        let img =
            SilvestreImage::new(vec![100, 150, 200], 1, 1, ColorSpace::Rgb).unwrap();
        let out = ResizeFilter::new(2, 2, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
        assert_eq!(out.color_space(), ColorSpace::Rgb);
        // Every pixel must equal the single source pixel.
        assert_eq!(out.pixels(), &[100, 150, 200, 100, 150, 200, 100, 150, 200, 100, 150, 200]);
    }

    #[test]
    fn bilinear_upscale_rgba_preserves_channels() {
        // 1×1 RGBA → 2×2; all output pixels equal the single source pixel.
        let img =
            SilvestreImage::new(vec![10, 20, 30, 255], 1, 1, ColorSpace::Rgba).unwrap();
        let out = ResizeFilter::new(2, 2, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.color_space(), ColorSpace::Rgba);
        assert_eq!(out.pixels().len(), 2 * 2 * 4);
        for chunk in out.pixels().chunks(4) {
            assert_eq!(chunk, &[10, 20, 30, 255]);
        }
    }

    // ------------------------------------------------------------------ //
    // Output dimensions are exact
    // ------------------------------------------------------------------ //

    #[test]
    fn output_dimensions_are_exact_nearest() {
        let img = gray(7, 3, vec![128; 21]);
        let out = ResizeFilter::new(5, 11, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 5);
        assert_eq!(out.height(), 11);
        assert_eq!(out.pixels().len(), 5 * 11);
    }

    #[test]
    fn output_dimensions_are_exact_bilinear() {
        let img = gray(7, 3, vec![128; 21]);
        let out = ResizeFilter::new(5, 11, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 5);
        assert_eq!(out.height(), 11);
        assert_eq!(out.pixels().len(), 5 * 11);
    }

    // ------------------------------------------------------------------ //
    // Edge cases
    // ------------------------------------------------------------------ //

    #[test]
    fn zero_target_width_returns_empty() {
        let img = gray(4, 4, vec![0; 16]);
        let out = ResizeFilter::new(0, 4, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 0);
        assert_eq!(out.height(), 0);
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn zero_target_height_returns_empty() {
        let img = gray(4, 4, vec![0; 16]);
        let out = ResizeFilter::new(4, 0, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        assert_eq!(out.width(), 0);
        assert_eq!(out.height(), 0);
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn empty_source_returns_empty() {
        let img = gray(0, 0, vec![]);
        for interp in [Interpolation::NearestNeighbor, Interpolation::Bilinear] {
            let out = ResizeFilter::new(4, 4, interp).apply(&img).unwrap();
            assert_eq!(out.width(), 0, "{interp:?}");
            assert_eq!(out.height(), 0, "{interp:?}");
            assert!(out.pixels().is_empty(), "{interp:?}");
        }
    }

    #[test]
    fn single_pixel_upscale_nearest() {
        let img = gray(1, 1, vec![77]);
        let out = ResizeFilter::new(3, 3, Interpolation::NearestNeighbor)
            .apply(&img)
            .unwrap();
        assert!(out.pixels().iter().all(|&v| v == 77));
    }

    #[test]
    fn single_pixel_upscale_bilinear() {
        let img = gray(1, 1, vec![77]);
        let out = ResizeFilter::new(3, 3, Interpolation::Bilinear)
            .apply(&img)
            .unwrap();
        // Single source pixel, no neighbors to blend with.
        assert!(out.pixels().iter().all(|&v| v == 77));
    }

    // ------------------------------------------------------------------ //
    // Accessors and Filter trait
    // ------------------------------------------------------------------ //

    #[test]
    fn accessors() {
        let f = ResizeFilter::new(10, 20, Interpolation::Bilinear);
        assert_eq!(f.target_width(), 10);
        assert_eq!(f.target_height(), 20);
        assert_eq!(f.interpolation(), Interpolation::Bilinear);
    }

    #[test]
    fn filter_trait_object_nearest() {
        let filter: Box<dyn Filter> = Box::new(ResizeFilter::new(2, 2, Interpolation::NearestNeighbor));
        let img = gray(1, 1, vec![42]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
        assert!(out.pixels().iter().all(|&v| v == 42));
    }

    #[test]
    fn filter_trait_object_bilinear() {
        let filter: Box<dyn Filter> = Box::new(ResizeFilter::new(2, 2, Interpolation::Bilinear));
        let img = gray(1, 1, vec![42]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
        assert!(out.pixels().iter().all(|&v| v == 42));
    }
}

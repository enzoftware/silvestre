//! Mirror / flip geometric transformation.
//!
//! Flips an image along the horizontal axis, the vertical axis, or both.
//!
//! # Quick start
//!
//! ```
//! use silvestre_core::transform::mirror::{MirrorFilter, MirrorMode};
//! use silvestre_core::filters::Filter;
//! use silvestre_core::{ColorSpace, SilvestreImage};
//!
//! // 2×1 RGB image: [R, G] (left-to-right)
//! let img = SilvestreImage::new(
//!     vec![255, 0, 0,  0, 255, 0],
//!     2, 1,
//!     ColorSpace::Rgb,
//! )?;
//! let flipped = MirrorFilter::new(MirrorMode::Horizontal).apply(&img)?;
//! // After horizontal flip the green pixel is on the left.
//! assert_eq!(flipped.get_pixel(0, 0)?, &[0, 255, 0]);
//! assert_eq!(flipped.get_pixel(1, 0)?, &[255, 0, 0]);
//! # Ok::<_, silvestre_core::SilvestreError>(())
//! ```

use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Axis along which the image is mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorMode {
    /// Flip left-to-right (mirror along the vertical axis).
    Horizontal,
    /// Flip top-to-bottom (mirror along the horizontal axis).
    Vertical,
    /// Apply both horizontal and vertical flips (equivalent to a 180° rotation).
    Both,
}

/// Geometric mirror / flip filter.
///
/// Produces a new image that is the reflection of the source along the
/// configured axis. The original image is left unchanged.
///
/// # Examples
///
/// ```
/// use silvestre_core::transform::mirror::{MirrorFilter, MirrorMode};
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let pixels = vec![
///     1, 2, 3,   // row 0: grayscale pixels A, B, C
///     4, 5, 6,   // row 1: pixels D, E, F
/// ];
/// let img = SilvestreImage::new(pixels, 3, 2, ColorSpace::Grayscale)?;
///
/// // Vertical flip swaps row 0 and row 1.
/// let vflip = MirrorFilter::new(MirrorMode::Vertical).apply(&img)?;
/// assert_eq!(vflip.pixels(), &[4, 5, 6, 1, 2, 3]);
///
/// // Horizontal flip reverses pixels within each row.
/// let hflip = MirrorFilter::new(MirrorMode::Horizontal).apply(&img)?;
/// assert_eq!(hflip.pixels(), &[3, 2, 1, 6, 5, 4]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirrorFilter {
    mode: MirrorMode,
}

impl MirrorFilter {
    /// Create a new `MirrorFilter` for the given [`MirrorMode`].
    #[must_use]
    pub fn new(mode: MirrorMode) -> Self {
        Self { mode }
    }

    /// The mirroring mode this filter applies.
    #[must_use]
    pub fn mode(&self) -> MirrorMode {
        self.mode
    }
}

impl Filter for MirrorFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let width = image.width() as usize;
        let height = image.height() as usize;
        let channels = image.color_space().channels();
        let src = image.pixels();

        if width == 0 || height == 0 {
            return SilvestreImage::new(
                src.to_vec(),
                image.width(),
                image.height(),
                image.color_space(),
            );
        }

        let row_bytes = width * channels;
        let mut dst = vec![0u8; src.len()];

        for y in 0..height {
            let src_row = match self.mode {
                MirrorMode::Horizontal => y,
                MirrorMode::Vertical | MirrorMode::Both => height - 1 - y,
            };

            let src_slice = &src[src_row * row_bytes..(src_row + 1) * row_bytes];
            let dst_slice = &mut dst[y * row_bytes..(y + 1) * row_bytes];

            if matches!(self.mode, MirrorMode::Horizontal | MirrorMode::Both) {
                for x in 0..width {
                    let src_x = width - 1 - x;
                    let src_pixel = &src_slice[src_x * channels..(src_x + 1) * channels];
                    dst_slice[x * channels..(x + 1) * channels].copy_from_slice(src_pixel);
                }
            } else {
                dst_slice.copy_from_slice(src_slice);
            }
        }

        SilvestreImage::new(dst, image.width(), image.height(), image.color_space())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSpace, SilvestreImage};

    fn gray(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    // ------------------------------------------------------------------ //
    // Horizontal flip
    // ------------------------------------------------------------------ //

    #[test]
    fn horizontal_reverses_pixels_in_each_row() {
        // 3×2 grayscale:
        //   row 0: 1 2 3
        //   row 1: 4 5 6
        let img = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let out = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
        assert_eq!(out.pixels(), &[3, 2, 1, 6, 5, 4]);
    }

    #[test]
    fn horizontal_single_column_is_identity() {
        let img = gray(1, 3, vec![10, 20, 30]);
        let out = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
        assert_eq!(out.pixels(), &[10, 20, 30]);
    }

    #[test]
    fn horizontal_round_trip() {
        let pixels: Vec<u8> = (0..12).collect();
        let img = gray(4, 3, pixels.clone());
        let filter = MirrorFilter::new(MirrorMode::Horizontal);
        let once = filter.apply(&img).unwrap();
        let twice = filter.apply(&once).unwrap();
        assert_eq!(twice.pixels(), img.pixels());
    }

    // ------------------------------------------------------------------ //
    // Vertical flip
    // ------------------------------------------------------------------ //

    #[test]
    fn vertical_swaps_rows() {
        let img = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let out = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
        assert_eq!(out.pixels(), &[4, 5, 6, 1, 2, 3]);
    }

    #[test]
    fn vertical_single_row_is_identity() {
        let img = gray(4, 1, vec![10, 20, 30, 40]);
        let out = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
        assert_eq!(out.pixels(), &[10, 20, 30, 40]);
    }

    #[test]
    fn vertical_round_trip() {
        let pixels: Vec<u8> = (0..15).collect();
        let img = gray(5, 3, pixels.clone());
        let filter = MirrorFilter::new(MirrorMode::Vertical);
        let once = filter.apply(&img).unwrap();
        let twice = filter.apply(&once).unwrap();
        assert_eq!(twice.pixels(), img.pixels());
    }

    // ------------------------------------------------------------------ //
    // Both flips
    // ------------------------------------------------------------------ //

    #[test]
    fn both_is_180_degree_rotation() {
        // 3×2 image: both = horizontal + vertical
        let img = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let out = MirrorFilter::new(MirrorMode::Both).apply(&img).unwrap();
        // Row 1 reversed then placed first, Row 0 reversed placed second.
        assert_eq!(out.pixels(), &[6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn both_round_trip() {
        let pixels: Vec<u8> = (0..20).collect();
        let img = gray(5, 4, pixels.clone());
        let filter = MirrorFilter::new(MirrorMode::Both);
        let once = filter.apply(&img).unwrap();
        let twice = filter.apply(&once).unwrap();
        assert_eq!(twice.pixels(), img.pixels());
    }

    #[test]
    fn both_equals_horizontal_then_vertical() {
        let pixels: Vec<u8> = (0..12).collect();
        let img = gray(4, 3, pixels);
        let both = MirrorFilter::new(MirrorMode::Both).apply(&img).unwrap();
        let h_then_v = MirrorFilter::new(MirrorMode::Vertical)
            .apply(&MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap())
            .unwrap();
        assert_eq!(both.pixels(), h_then_v.pixels());
    }

    // ------------------------------------------------------------------ //
    // Multi-channel images
    // ------------------------------------------------------------------ //

    #[test]
    fn horizontal_flip_rgba() {
        // 2×1 RGBA: [R=255,G=0,B=0,A=255] | [R=0,G=255,B=0,A=255]
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let img = SilvestreImage::new(pixels, 2, 1, ColorSpace::Rgba).unwrap();
        let out = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
        assert_eq!(out.pixels(), &[0, 255, 0, 255, 255, 0, 0, 255]);
    }

    #[test]
    fn vertical_flip_rgb() {
        // 2×2 RGB
        // row 0: Red  | Green
        // row 1: Blue | White
        let pixels = vec![
            255, 0, 0, 0, 255, 0, // row 0
            0, 0, 255, 255, 255, 255, // row 1
        ];
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgb).unwrap();
        let out = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
        assert_eq!(
            out.pixels(),
            &[0, 0, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0]
        );
    }

    // ------------------------------------------------------------------ //
    // Asymmetric image verification
    // ------------------------------------------------------------------ //

    #[test]
    fn asymmetric_image_horizontal() {
        // Asymmetric 3×3 grayscale — verify pixel positions precisely.
        //   Original:        After H-flip:
        //   1  2  3          3  2  1
        //   4  5  6    =>    6  5  4
        //   7  8  9          9  8  7
        let img = gray(3, 3, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let out = MirrorFilter::new(MirrorMode::Horizontal).apply(&img).unwrap();
        assert_eq!(out.get_pixel(0, 0).unwrap(), &[3]);
        assert_eq!(out.get_pixel(2, 0).unwrap(), &[1]);
        assert_eq!(out.get_pixel(0, 1).unwrap(), &[6]);
        assert_eq!(out.get_pixel(1, 1).unwrap(), &[5]); // center stays
        assert_eq!(out.get_pixel(2, 2).unwrap(), &[7]);
    }

    #[test]
    fn asymmetric_image_vertical() {
        //   Original:        After V-flip:
        //   1  2  3          7  8  9
        //   4  5  6    =>    4  5  6
        //   7  8  9          1  2  3
        let img = gray(3, 3, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let out = MirrorFilter::new(MirrorMode::Vertical).apply(&img).unwrap();
        assert_eq!(out.get_pixel(0, 0).unwrap(), &[7]);
        assert_eq!(out.get_pixel(2, 0).unwrap(), &[9]);
        assert_eq!(out.get_pixel(1, 1).unwrap(), &[5]); // center stays
        assert_eq!(out.get_pixel(0, 2).unwrap(), &[1]);
    }

    // ------------------------------------------------------------------ //
    // Metadata preservation
    // ------------------------------------------------------------------ //

    #[test]
    fn preserves_dimensions_and_color_space() {
        let img = SilvestreImage::new(vec![0; 4 * 3 * 4], 4, 3, ColorSpace::Rgba).unwrap();
        for mode in [MirrorMode::Horizontal, MirrorMode::Vertical, MirrorMode::Both] {
            let out = MirrorFilter::new(mode).apply(&img).unwrap();
            assert_eq!(out.width(), 4, "{mode:?}");
            assert_eq!(out.height(), 3, "{mode:?}");
            assert_eq!(out.color_space(), ColorSpace::Rgba, "{mode:?}");
        }
    }

    // ------------------------------------------------------------------ //
    // Edge cases
    // ------------------------------------------------------------------ //

    #[test]
    fn empty_image_is_preserved() {
        let img = gray(0, 0, vec![]);
        for mode in [MirrorMode::Horizontal, MirrorMode::Vertical, MirrorMode::Both] {
            let out = MirrorFilter::new(mode).apply(&img).unwrap();
            assert_eq!(out.width(), 0, "{mode:?}");
            assert_eq!(out.height(), 0, "{mode:?}");
            assert!(out.pixels().is_empty(), "{mode:?}");
        }
    }

    #[test]
    fn single_pixel_is_identity() {
        let img = gray(1, 1, vec![42]);
        for mode in [MirrorMode::Horizontal, MirrorMode::Vertical, MirrorMode::Both] {
            let out = MirrorFilter::new(mode).apply(&img).unwrap();
            assert_eq!(out.pixels(), &[42], "{mode:?}");
        }
    }

    #[test]
    fn uniform_image_unchanged_by_any_mode() {
        let img = gray(4, 4, vec![128; 16]);
        for mode in [MirrorMode::Horizontal, MirrorMode::Vertical, MirrorMode::Both] {
            let out = MirrorFilter::new(mode).apply(&img).unwrap();
            assert!(out.pixels().iter().all(|&v| v == 128), "{mode:?}");
        }
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(MirrorFilter::new(MirrorMode::Horizontal));
        let img = gray(2, 2, vec![1, 2, 3, 4]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels(), &[2, 1, 4, 3]);
    }
}

//! Crop geometric transformation.
//!
//! Extracts a rectangular sub-region from an image.
//!
//! # Quick start
//!
//! ```
//! use silvestre_core::transform::crop::CropFilter;
//! use silvestre_core::filters::Filter;
//! use silvestre_core::{ColorSpace, SilvestreImage};
//!
//! // 3×3 grayscale image
//! let img = SilvestreImage::new(
//!     vec![
//!         1, 2, 3,
//!         4, 5, 6,
//!         7, 8, 9,
//!     ],
//!     3, 3,
//!     ColorSpace::Grayscale,
//! )?;
//!
//! // Crop the center 2×2 region starting at (0, 0)
//! let cropped = CropFilter::new(0, 0, 2, 2).apply(&img)?;
//! // Result is a 2×2 image with pixels: [1, 2, 4, 5]
//! assert_eq!(cropped.width(), 2);
//! assert_eq!(cropped.height(), 2);
//! assert_eq!(cropped.pixels(), &[1, 2, 4, 5]);
//! # Ok::<_, silvestre_core::SilvestreError>(())
//! ```

use crate::filters::Filter;
use crate::{Result, SilvestreError, SilvestreImage};

/// Geometric crop filter.
///
/// Extracts a rectangular sub-region from the source image specified by
/// (x, y) coordinates and width/height dimensions. The original image is
/// left unchanged.
///
/// # Examples
///
/// ```
/// use silvestre_core::transform::crop::CropFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let pixels = vec![
///     1, 2, 3,   // row 0
///     4, 5, 6,   // row 1
///     7, 8, 9,   // row 2
/// ];
/// let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Grayscale)?;
///
/// // Crop a 2×2 region starting at (1, 1) - should give [5, 6, 8, 9]
/// let cropped = CropFilter::new(1, 1, 2, 2).apply(&img)?;
/// assert_eq!(cropped.width(), 2);
/// assert_eq!(cropped.height(), 2);
/// assert_eq!(cropped.pixels(), &[5, 6, 8, 9]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CropFilter {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl CropFilter {
    /// Create a new `CropFilter` for the given region.
    ///
    /// # Arguments
    ///
    /// * `x` - Left coordinate of the crop region
    /// * `y` - Top coordinate of the crop region
    /// * `width` - Width of the crop region
    /// * `height` - Height of the crop region
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    /// The left coordinate of the crop region.
    #[must_use]
    pub const fn x(&self) -> u32 {
        self.x
    }

    /// The top coordinate of the crop region.
    #[must_use]
    pub const fn y(&self) -> u32 {
        self.y
    }

    /// The width of the crop region.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// The height of the crop region.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

impl Filter for CropFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        // Validate crop dimensions
        if self.width == 0 || self.height == 0 {
            return Err(SilvestreError::InvalidParameter(
                "crop dimensions must be non-zero".to_string(),
            ));
        }

        // Validate that crop region is within bounds
        let x_end = self.x.checked_add(self.width).ok_or_else(|| {
            SilvestreError::InvalidParameter(
                "crop x + width overflows u32".to_string(),
            )
        })?;
        let y_end = self.y.checked_add(self.height).ok_or_else(|| {
            SilvestreError::InvalidParameter(
                "crop y + height overflows u32".to_string(),
            )
        })?;

        if x_end > image.width() || y_end > image.height() {
            return Err(SilvestreError::InvalidParameter(
                format!(
                    "crop region ({}, {}, {}, {}) exceeds image bounds ({}x{})",
                    self.x, self.y, self.width, self.height,
                    image.width(), image.height()
                ),
            ));
        }

        let channels = image.color_space().channels();
        let src = image.pixels();
        let src_width = image.width() as usize;

        let crop_width = self.width as usize;
        let crop_height = self.height as usize;
        let start_x = self.x as usize;
        let start_y = self.y as usize;

        let row_bytes = crop_width * channels;
        let mut dst = vec![0u8; row_bytes * crop_height];

        for y in 0..crop_height {
            let src_y = start_y + y;
            let src_row_start = src_y * src_width * channels;
            let dst_row_start = y * crop_width * channels;

            for x in 0..crop_width {
                let src_x = start_x + x;
                let src_pixel_start = src_row_start + src_x * channels;
                let dst_pixel_start = dst_row_start + x * channels;

                dst[dst_pixel_start..dst_pixel_start + channels]
                    .copy_from_slice(&src[src_pixel_start..src_pixel_start + channels]);
            }
        }

        SilvestreImage::new(dst, self.width, self.height, image.color_space())
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
    // Full image crop
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_full_image_returns_identical_copy() {
        let pixels = vec![1, 2, 3, 4, 5, 6];
        let img = gray(3, 2, pixels.clone());
        let cropped = CropFilter::new(0, 0, 3, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 3);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixels(), &pixels);
    }

    #[test]
    fn crop_full_single_pixel() {
        let img = gray(1, 1, vec![42]);
        let cropped = CropFilter::new(0, 0, 1, 1).apply(&img).unwrap();
        assert_eq!(cropped.pixels(), &[42]);
    }

    // ------------------------------------------------------------------ //
    // Sub-region crops
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_top_left_corner() {
        let pixels = vec![
            1, 2, 3, // row 0
            4, 5, 6, // row 1
            7, 8, 9, // row 2
        ];
        let img = gray(3, 3, pixels);
        let cropped = CropFilter::new(0, 0, 2, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 2);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixels(), &[1, 2, 4, 5]);
    }

    #[test]
    fn crop_bottom_right_corner() {
        let pixels = vec![
            1, 2, 3, // row 0
            4, 5, 6, // row 1
            7, 8, 9, // row 2
        ];
        let img = gray(3, 3, pixels);
        let cropped = CropFilter::new(1, 1, 2, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 2);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixels(), &[5, 6, 8, 9]);
    }

    #[test]
    fn crop_center_region() {
        let pixels = vec![
            1, 2, 3, 4, // row 0
            5, 6, 7, 8, // row 1
            9, 10, 11, 12, // row 2
            13, 14, 15, 16, // row 3
        ];
        let img = gray(4, 4, pixels);
        let cropped = CropFilter::new(1, 1, 2, 2).apply(&img).unwrap();
        assert_eq!(cropped.pixels(), &[6, 7, 10, 11]);
    }

    #[test]
    fn crop_single_row() {
        let pixels = vec![1, 2, 3, 4, 5];
        let img = gray(5, 1, pixels);
        let cropped = CropFilter::new(1, 0, 3, 1).apply(&img).unwrap();
        assert_eq!(cropped.width(), 3);
        assert_eq!(cropped.height(), 1);
        assert_eq!(cropped.pixels(), &[2, 3, 4]);
    }

    #[test]
    fn crop_single_column() {
        let pixels = vec![1, 2, 3, 4, 5];
        let img = gray(1, 5, pixels);
        let cropped = CropFilter::new(0, 1, 1, 3).apply(&img).unwrap();
        assert_eq!(cropped.width(), 1);
        assert_eq!(cropped.height(), 3);
        assert_eq!(cropped.pixels(), &[2, 3, 4]);
    }

    #[test]
    fn crop_middle_region_tall_image() {
        let pixels = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let img = gray(3, 4, pixels);
        let cropped = CropFilter::new(0, 1, 3, 2).apply(&img).unwrap();
        assert_eq!(cropped.pixels(), &[4, 5, 6, 7, 8, 9]);
    }

    // ------------------------------------------------------------------ //
    // Out-of-bounds validation
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_x_exceeds_bounds() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(2, 0, 2, 1).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_y_exceeds_bounds() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 2, 1, 2).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_width_exceeds_bounds() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 0, 4, 1).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_height_exceeds_bounds() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 0, 1, 4).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_x_plus_width_exceeds_bounds() {
        let img = gray(5, 5, vec![0; 25]);
        let result = CropFilter::new(3, 0, 3, 1).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_y_plus_height_exceeds_bounds() {
        let img = gray(5, 5, vec![0; 25]);
        let result = CropFilter::new(0, 3, 1, 3).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_at_exact_boundary() {
        let img = gray(5, 5, vec![0; 25]);
        // Crop exactly to the boundary should work
        let cropped = CropFilter::new(3, 3, 2, 2).apply(&img);
        assert!(cropped.is_ok());
    }

    #[test]
    fn crop_beyond_boundary() {
        let img = gray(5, 5, vec![0; 25]);
        // One pixel beyond the boundary should fail
        let result = CropFilter::new(3, 3, 3, 2).apply(&img);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------ //
    // Zero-dimension validation
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_zero_width_returns_error() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 0, 0, 1).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_zero_height_returns_error() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 0, 1, 0).apply(&img);
        assert!(result.is_err());
    }

    #[test]
    fn crop_zero_both_dimensions_returns_error() {
        let img = gray(3, 3, vec![0; 9]);
        let result = CropFilter::new(0, 0, 0, 0).apply(&img);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------ //
    // Multi-channel images
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_rgb_image() {
        // 2×2 RGB image
        // [R, G, B] | [R, G, B]
        // [R, G, B] | [R, G, B]
        let pixels = vec![
            255, 0, 0, 0, 255, 0, // row 0: red, green
            0, 0, 255, 255, 255, 255, // row 1: blue, white
        ];
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgb).unwrap();
        let cropped = CropFilter::new(1, 0, 1, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 1);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixels(), &[0, 255, 0, 255, 255, 255]);
    }

    #[test]
    fn crop_rgba_image() {
        // 2×2 RGBA image
        let pixels = vec![
            255, 0, 0, 255, 0, 255, 0, 255, // row 0: red, green
            0, 0, 255, 255, 255, 255, 255, 255, // row 1: blue, white
        ];
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgba).unwrap();
        let cropped = CropFilter::new(0, 0, 1, 2).apply(&img).unwrap();
        assert_eq!(cropped.pixels(), &[255, 0, 0, 255, 0, 0, 255, 255]);
    }

    // ------------------------------------------------------------------ //
    // Metadata preservation
    // ------------------------------------------------------------------ //

    #[test]
    fn preserves_color_space() {
        let img = SilvestreImage::new(vec![0; 2 * 2 * 3], 2, 2, ColorSpace::Rgb).unwrap();
        let cropped = CropFilter::new(0, 0, 1, 1).apply(&img).unwrap();
        assert_eq!(cropped.color_space(), ColorSpace::Rgb);
    }

    #[test]
    fn preserves_dimensions() {
        let img = gray(5, 5, vec![0; 25]);
        let cropped = CropFilter::new(1, 1, 3, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 3);
        assert_eq!(cropped.height(), 2);
    }

    // ------------------------------------------------------------------ //
    // Edge cases
    // ------------------------------------------------------------------ //

    #[test]
    fn crop_1x1_from_large_image() {
        let pixels: Vec<u8> = (0..100).collect();
        let img = gray(10, 10, pixels);
        let cropped = CropFilter::new(5, 5, 1, 1).apply(&img).unwrap();
        // At position (5, 5) in 10-wide image: offset = 5 * 10 + 5 = 55
        assert_eq!(cropped.pixels(), &[55]);
    }

    #[test]
    fn crop_asymmetric_region() {
        let pixels = vec![
            1, 2, 3, 4, 5,    // row 0
            6, 7, 8, 9, 10,   // row 1
            11, 12, 13, 14, 15, // row 2
        ];
        let img = gray(5, 3, pixels);
        let cropped = CropFilter::new(1, 0, 3, 2).apply(&img).unwrap();
        assert_eq!(cropped.width(), 3);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.pixels(), &[2, 3, 4, 7, 8, 9]);
    }

    // ------------------------------------------------------------------ //
    // Trait object
    // ------------------------------------------------------------------ //

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(CropFilter::new(0, 0, 2, 2));
        let img = gray(3, 3, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels(), &[1, 2, 4, 5]);
    }
}

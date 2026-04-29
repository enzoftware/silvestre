//! Sharpen filter.
//!
//! Enhances edges in an image using a Laplacian-based kernel.

use crate::filters::convolution::{apply_kernel, BorderMode, Kernel};
use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Sharpen filter using a Laplacian-based 3x3 kernel.
///
/// This filter enhances edges by subtracting the Laplacian (second derivative)
/// from the original image.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{Filter, SharpenFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale).unwrap();
/// let filter = SharpenFilter::new().unwrap();
/// let out = filter.apply(&img).unwrap();
/// assert_eq!(out.width(), 5);
/// ```
#[derive(Debug, Clone)]
pub struct SharpenFilter {
    kernel: Kernel,
    border: BorderMode,
}

impl SharpenFilter {
    /// Create a new sharpen filter with default border mode (Clamp).
    /// 
    /// Returns an error if the kernel cannot be created.
    pub fn new() -> Result<Self> {
        Self::with_border(BorderMode::Clamp)
    }

    /// Create a new sharpen filter with a specific border mode.
    /// 
    /// Returns an error if the kernel cannot be created.
    pub fn with_border(border: BorderMode) -> Result<Self> {
        let kernel = Kernel::square(
            vec![
                 0.0, -1.0,  0.0,
                -1.0,  5.0, -1.0,
                 0.0, -1.0,  0.0,
            ],
            3,
        )?;
        Ok(Self { kernel, border })
    }
}

impl Default for SharpenFilter {
    fn default() -> Self {
        Self::new().expect("Failed to create default SharpenFilter")
    }
}

impl Filter for SharpenFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        apply_kernel(image, &self.kernel, self.border)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    #[test]
    fn sharpen_enhances_contrast() {
        let pixels = vec![100, 100, 100, 100, 200, 100, 100, 100, 100];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Grayscale).unwrap();
        let filter = SharpenFilter::new().unwrap();
        let out = filter.apply(&img).unwrap();
        // The center pixel (200) has lower-valued neighbors (100). 
        // Sharpening should push it higher.
        assert!(out.pixels()[4] > 200);
    }

    #[test]
    fn sharpen_grayscale_image() {
        let pixels = vec![0, 0, 0, 0, 255, 0, 0, 0, 0];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Grayscale).unwrap();
        let filter = SharpenFilter::new().unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 3);
        assert_eq!(out.height(), 3);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        // Center pixel calculation: 255 * 5 - 4 * 0 = 1275, clamped to 255.
        assert_eq!(out.pixels()[4], 255);
    }

    #[test]
    fn sharpen_rgb_image() {
        let pixels = vec![
            0, 0, 0,  0, 0, 0,  0, 0, 0,
            0, 0, 0,  100, 100, 100,  0, 0, 0,
            0, 0, 0,  0, 0, 0,  0, 0, 0,
        ];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Rgb).unwrap();
        let filter = SharpenFilter::new().unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Rgb);
        // Center pixel (100, 100, 100) should be enhanced.
        // Index of center pixel (1,1) in RGB is 1 * (3*3) + 1*3 = 12, 13, 14
        assert!(out.pixels()[12] > 100);
        assert!(out.pixels()[13] > 100);
        assert!(out.pixels()[14] > 100);
    }

    #[test]
    fn sharpen_uniform_image_preserves_values() {
        let pixels = vec![100; 9];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Grayscale).unwrap();
        let filter = SharpenFilter::new().unwrap();
        let out = filter.apply(&img).unwrap();
        // Kernel sum is 1.0, so uniform area should remain uniform.
        assert_eq!(out.pixels()[4], 100);
    }

    #[test]
    fn sharpen_with_different_border_modes() {
        let pixels = vec![100; 9];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Grayscale).unwrap();
        
        // Zero border padding.
        let filter = SharpenFilter::with_border(BorderMode::Zero).unwrap();
        let out = filter.apply(&img).unwrap();
        // Edge pixels will change because they see the 0 border.
        // (0,0) pixel sees: 0 0 0; 0 100 100; 0 100 100
        // Kernel: 0 -1 0; -1 5 -1; 0 -1 0
        // (0,0) with Zero: 100*5 - 100(right) - 100(bottom) - 0(left) - 0(top) = 300
        assert_eq!(out.pixels()[0], 255); // Clamped to 255
    }
}

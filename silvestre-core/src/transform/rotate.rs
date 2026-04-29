//! Image rotation transform.
//!
//! Supports fixed rotations (90°, 180°, 270°) with optimized lossless paths
//! and arbitrary angle rotation using bilinear interpolation.

use crate::filters::Filter;
use crate::{Result, SilvestreError, SilvestreImage};

/// Rotation filter for fixed and arbitrary angles.
///
/// - Fixed rotations (90°, 180°, 270°) are fast and lossless.
/// - Arbitrary angles use bilinear interpolation and may lose precision at boundaries.
/// - Output canvas maintains input dimensions (except 90°/270° swap width/height).
/// - Background color is applied to out-of-bounds pixels during arbitrary rotation.
///
/// # Examples
///
/// ```
/// use silvestre_core::transform::rotate::RotateFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(
///     vec![1, 2, 3, 4],
///     2, 2,
///     ColorSpace::Grayscale,
/// )?;
/// let rotated = RotateFilter::new(90.0, 255, [255, 255, 255]).apply(&img)?;
/// assert_eq!(rotated.width(), 2);  // Height becomes width for 90°
/// assert_eq!(rotated.height(), 2);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotateFilter {
    angle: f64,
    grayscale_background: u8,
    rgb_background: [u8; 3],
}

impl RotateFilter {
    /// Create a new rotation filter.
    ///
    /// # Arguments
    /// - `angle`: Rotation angle in degrees (0-360, normalized automatically)
    /// - `grayscale_background`: Background value for grayscale images (0-255)
    /// - `rgb_background`: Background RGB values for color images
    #[must_use]
    pub fn new(angle: f64, grayscale_background: u8, rgb_background: [u8; 3]) -> Self {
        Self {
            angle,
            grayscale_background,
            rgb_background,
        }
    }

    /// The rotation angle in degrees.
    #[must_use]
    pub fn angle(&self) -> f64 {
        self.angle
    }

    /// Background color value for grayscale images.
    #[must_use]
    pub fn grayscale_background(&self) -> u8 {
        self.grayscale_background
    }

    /// Background RGB values for color images.
    #[must_use]
    pub fn rgb_background(&self) -> [u8; 3] {
        self.rgb_background
    }

    /// Normalize angle to 0-360 range.
    fn normalize_angle(angle: f64) -> f64 {
        let mut a = angle % 360.0;
        if a < 0.0 {
            a += 360.0;
        }
        a
    }

    /// Check if angle is close to a fixed angle (within 1e-6 tolerance).
    fn is_fixed_angle(angle: f64) -> Option<u16> {
        let normalized = Self::normalize_angle(angle);
        const TOLERANCE: f64 = 1e-6;

        if (normalized - 0.0).abs() < TOLERANCE {
            Some(0)
        } else if (normalized - 90.0).abs() < TOLERANCE {
            Some(90)
        } else if (normalized - 180.0).abs() < TOLERANCE {
            Some(180)
        } else if (normalized - 270.0).abs() < TOLERANCE {
            Some(270)
        } else {
            None
        }
    }
}

impl Filter for RotateFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        // Placeholder - will implement in later tasks
        todo!("Implement rotation apply method")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    // Tests will be added in following tasks
}

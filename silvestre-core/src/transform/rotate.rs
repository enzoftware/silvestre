//! Image rotation transform.
//!
//! Supports fixed rotations (90°, 180°, 270°) with optimized lossless paths
//! and arbitrary angle rotation using bilinear interpolation.

use crate::filters::Filter;
use crate::{Result, SilvestreImage};

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
        let normalized_angle = Self::normalize_angle(self.angle);

        // Handle empty image
        if image.width() == 0 || image.height() == 0 {
            return SilvestreImage::new(
                image.pixels().to_vec(),
                image.width(),
                image.height(),
                image.color_space(),
            );
        }

        // Check if it's a fixed angle
        if let Some(fixed_angle) = Self::is_fixed_angle(normalized_angle) {
            match fixed_angle {
                0 => self.rotate_0(image),
                90 => self.rotate_90(image),
                180 => self.rotate_180(image),
                270 => self.rotate_270(image),
                _ => unreachable!(),
            }
        } else {
            self.rotate_arbitrary(image)
        }
    }
}

impl RotateFilter {
    /// 0° rotation: identity, return a clone.
    fn rotate_0(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        SilvestreImage::new(
            image.pixels().to_vec(),
            image.width(),
            image.height(),
            image.color_space(),
        )
    }

    /// 90° rotation: placeholder for next task.
    fn rotate_90(&self, _image: &SilvestreImage) -> Result<SilvestreImage> {
        todo!("Implement 90° rotation")
    }

    /// 180° rotation: equivalent to mirroring both horizontally and vertically.
    fn rotate_180(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let src = image.pixels();
        let mut dst = vec![0u8; src.len()];
        for (i, pixel) in src.iter().enumerate() {
            dst[src.len() - 1 - i] = *pixel;
        }
        SilvestreImage::new(dst, image.width(), image.height(), image.color_space())
    }

    /// 270° rotation: placeholder for next task.
    fn rotate_270(&self, _image: &SilvestreImage) -> Result<SilvestreImage> {
        todo!("Implement 270° rotation")
    }

    /// Arbitrary angle rotation with bilinear interpolation.
    fn rotate_arbitrary(&self, _image: &SilvestreImage) -> Result<SilvestreImage> {
        todo!("Implement arbitrary angle rotation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn rotate_0_degrees_returns_clone() {
        let pixels = vec![1, 2, 3, 4, 5, 6];
        let img = gray(3, 2, pixels.clone());
        let rotated = RotateFilter::new(0.0, 255, [255, 255, 255]).apply(&img).unwrap();
        assert_eq!(rotated.pixels(), &pixels);
        assert_eq!(rotated.width(), img.width());
        assert_eq!(rotated.height(), img.height());
    }

    #[test]
    fn rotate_360_degrees_returns_clone() {
        let pixels = vec![1, 2, 3, 4, 5, 6];
        let img = gray(3, 2, pixels.clone());
        let rotated = RotateFilter::new(360.0, 255, [255, 255, 255]).apply(&img).unwrap();
        assert_eq!(rotated.pixels(), &pixels);
    }

    #[test]
    fn rotate_negative_angle_normalizes() {
        let pixels = vec![1, 2, 3, 4];
        let img = gray(2, 2, pixels.clone());
        let rotated = RotateFilter::new(-360.0, 255, [255, 255, 255]).apply(&img).unwrap();
        assert_eq!(rotated.pixels(), &pixels);
    }

    // Task 3: 180° Rotation Tests
    #[test]
    fn rotate_180_degrees_grayscale() {
        let img = gray(3, 2, vec![1, 2, 3, 4, 5, 6]);
        let rotated = RotateFilter::new(180.0, 255, [255, 255, 255])
            .apply(&img)
            .unwrap();
        assert_eq!(rotated.pixels(), &[6, 5, 4, 3, 2, 1]);
        assert_eq!(rotated.width(), 3);
        assert_eq!(rotated.height(), 2);
    }

    #[test]
    fn rotate_180_round_trip() {
        let pixels: Vec<u8> = (0..12).collect();
        let img = gray(4, 3, pixels.clone());
        let filter = RotateFilter::new(180.0, 255, [255, 255, 255]);
        let once = filter.apply(&img).unwrap();
        let twice = filter.apply(&once).unwrap();
        assert_eq!(twice.pixels(), &pixels);
    }

    #[test]
    fn rotate_180_equals_mirror_both() {
        let pixels: Vec<u8> = (0..12).collect();
        let img = gray(4, 3, pixels);
        let rotated = RotateFilter::new(180.0, 255, [255, 255, 255])
            .apply(&img)
            .unwrap();
        let mirror_both = crate::transform::MirrorFilter::new(crate::transform::MirrorMode::Both)
            .apply(&img)
            .unwrap();
        assert_eq!(rotated.pixels(), mirror_both.pixels());
    }
}

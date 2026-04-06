use crate::{Result, SilvestreError};

/// Supported color space representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Rgba,
    Rgb,
    Grayscale,
}

impl ColorSpace {
    /// Number of bytes per pixel for this color space.
    #[must_use]
    pub const fn channels(self) -> usize {
        match self {
            Self::Rgba => 4,
            Self::Rgb => 3,
            Self::Grayscale => 1,
        }
    }
}

/// Core image buffer type. Pixels are stored in row-major order.
#[derive(Debug, Clone)]
pub struct SilvestreImage {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    color_space: ColorSpace,
}

impl SilvestreImage {
    /// Create a new image from raw pixel data.
    ///
    /// Returns an error if the buffer length does not match
    /// `width * height * color_space.channels()`.
    pub fn new(pixels: Vec<u8>, width: u32, height: u32, color_space: ColorSpace) -> Result<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(color_space.channels()))
            .ok_or(SilvestreError::InvalidDimensions { width, height })?;
        if pixels.len() != expected {
            return Err(SilvestreError::InvalidDimensions { width, height });
        }
        Ok(Self {
            pixels,
            width,
            height,
            color_space,
        })
    }

    /// Create an image filled with zeros.
    #[must_use]
    pub fn zeroed(width: u32, height: u32, color_space: ColorSpace) -> Self {
        let len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|v| v.checked_mul(color_space.channels()))
            .expect("image dimensions overflow");
        Self {
            pixels: vec![0; len],
            width,
            height,
            color_space,
        }
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[must_use]
    pub fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// Raw pixel data as a byte slice.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Mutable access to raw pixel data.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        &mut self.pixels
    }

    /// Get the pixel value at (x, y) as a slice of channels.
    ///
    /// # Panics
    /// Panics if (x, y) is out of bounds.
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> &[u8] {
        assert!(x < self.width && y < self.height, "pixel coordinates out of bounds");
        let channels = self.color_space.channels();
        let offset = (y as usize * self.width as usize + x as usize) * channels;
        &self.pixels[offset..offset + channels]
    }

    /// Set the pixel value at (x, y).
    ///
    /// # Panics
    /// Panics if (x, y) is out of bounds or `value.len()` does not match the channel count.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: &[u8]) {
        assert!(x < self.width && y < self.height, "pixel coordinates out of bounds");
        let channels = self.color_space.channels();
        assert_eq!(value.len(), channels);
        let offset = (y as usize * self.width as usize + x as usize) * channels;
        self.pixels[offset..offset + channels].copy_from_slice(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_rgba_image() {
        let img = SilvestreImage::new(vec![0; 4 * 2 * 2], 2, 2, ColorSpace::Rgba).unwrap();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 2);
        assert_eq!(img.color_space(), ColorSpace::Rgba);
        assert_eq!(img.pixels().len(), 16);
    }

    #[test]
    fn create_image_invalid_size() {
        let result = SilvestreImage::new(vec![0; 5], 2, 2, ColorSpace::Rgba);
        assert!(result.is_err());
    }

    #[test]
    fn pixel_access() {
        let mut img = SilvestreImage::zeroed(2, 2, ColorSpace::Rgb);
        img.set_pixel(1, 0, &[255, 128, 64]);
        assert_eq!(img.pixel(1, 0), &[255, 128, 64]);
        assert_eq!(img.pixel(0, 0), &[0, 0, 0]);
    }

    #[test]
    fn zeroed_image() {
        let img = SilvestreImage::zeroed(10, 10, ColorSpace::Grayscale);
        assert_eq!(img.pixels().len(), 100);
        assert!(img.pixels().iter().all(|&b| b == 0));
    }
}

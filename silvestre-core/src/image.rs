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
            return Err(SilvestreError::BufferSizeMismatch {
                expected,
                got: pixels.len(),
            });
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

    /// Create a new image from raw pixel data.
    ///
    /// This is an alias for [`SilvestreImage::new`] provided for clarity
    /// when constructing an image from an existing pixel buffer.
    pub fn from_raw_pixels(
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        color_space: ColorSpace,
    ) -> Result<Self> {
        Self::new(pixels, width, height, color_space)
    }

    /// Get the pixel value at (x, y) as a slice of channels.
    ///
    /// Returns an error if (x, y) is out of bounds.
    pub fn get_pixel(&self, x: u32, y: u32) -> Result<&[u8]> {
        if x >= self.width || y >= self.height {
            return Err(SilvestreError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        let channels = self.color_space.channels();
        let offset = (y as usize * self.width as usize + x as usize) * channels;
        Ok(&self.pixels[offset..offset + channels])
    }

    /// Set the pixel value at (x, y).
    ///
    /// Returns an error if (x, y) is out of bounds or if the value length
    /// does not match the number of channels for this image's color space.
    pub fn set_pixel(&mut self, x: u32, y: u32, value: &[u8]) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(SilvestreError::OutOfBounds {
                x,
                y,
                width: self.width,
                height: self.height,
            });
        }
        let channels = self.color_space.channels();
        if value.len() != channels {
            return Err(SilvestreError::ChannelMismatch {
                expected: channels,
                got: value.len(),
            });
        }
        let offset = (y as usize * self.width as usize + x as usize) * channels;
        self.pixels[offset..offset + channels].copy_from_slice(value);
        Ok(())
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
    fn create_rgb_image() {
        let img = SilvestreImage::new(vec![0; 3 * 3 * 2], 3, 2, ColorSpace::Rgb).unwrap();
        assert_eq!(img.width(), 3);
        assert_eq!(img.height(), 2);
        assert_eq!(img.color_space(), ColorSpace::Rgb);
        assert_eq!(img.pixels().len(), 18);
    }

    #[test]
    fn create_grayscale_image() {
        let img = SilvestreImage::new(vec![0; 4 * 4], 4, 4, ColorSpace::Grayscale).unwrap();
        assert_eq!(img.color_space(), ColorSpace::Grayscale);
        assert_eq!(img.pixels().len(), 16);
    }

    #[test]
    fn create_image_invalid_size() {
        let result = SilvestreImage::new(vec![0; 5], 2, 2, ColorSpace::Rgba);
        assert!(matches!(
            result,
            Err(SilvestreError::BufferSizeMismatch { expected: 16, got: 5 })
        ));
    }

    #[test]
    fn create_image_zero_dimensions() {
        let result = SilvestreImage::new(vec![], 0, 0, ColorSpace::Rgba);
        assert!(result.is_ok());
    }

    #[test]
    fn from_raw_pixels_constructs_correctly() {
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let img = SilvestreImage::from_raw_pixels(pixels.clone(), 2, 1, ColorSpace::Rgba).unwrap();
        assert_eq!(img.width(), 2);
        assert_eq!(img.height(), 1);
        assert_eq!(img.pixels(), &pixels);
    }

    #[test]
    fn from_raw_pixels_rejects_invalid_buffer() {
        let result = SilvestreImage::from_raw_pixels(vec![0; 7], 2, 1, ColorSpace::Rgba);
        assert!(matches!(
            result,
            Err(SilvestreError::BufferSizeMismatch { expected: 8, got: 7 })
        ));
    }

    #[test]
    fn pixel_access() {
        let mut img = SilvestreImage::zeroed(2, 2, ColorSpace::Rgb);
        img.set_pixel(1, 0, &[255, 128, 64]).unwrap();
        assert_eq!(img.get_pixel(1, 0).unwrap(), &[255, 128, 64]);
        assert_eq!(img.get_pixel(0, 0).unwrap(), &[0, 0, 0]);
    }

    #[test]
    fn get_pixel_out_of_bounds() {
        let img = SilvestreImage::zeroed(2, 2, ColorSpace::Rgba);
        assert!(matches!(
            img.get_pixel(2, 0),
            Err(SilvestreError::OutOfBounds { x: 2, y: 0, width: 2, height: 2 })
        ));
        assert!(matches!(
            img.get_pixel(0, 2),
            Err(SilvestreError::OutOfBounds { x: 0, y: 2, width: 2, height: 2 })
        ));
        assert!(matches!(
            img.get_pixel(2, 2),
            Err(SilvestreError::OutOfBounds { x: 2, y: 2, width: 2, height: 2 })
        ));
    }

    #[test]
    fn set_pixel_out_of_bounds() {
        let mut img = SilvestreImage::zeroed(2, 2, ColorSpace::Rgba);
        assert!(matches!(
            img.set_pixel(2, 0, &[0, 0, 0, 0]),
            Err(SilvestreError::OutOfBounds { x: 2, y: 0, width: 2, height: 2 })
        ));
        assert!(matches!(
            img.set_pixel(0, 2, &[0, 0, 0, 0]),
            Err(SilvestreError::OutOfBounds { x: 0, y: 2, width: 2, height: 2 })
        ));
    }

    #[test]
    fn set_pixel_wrong_channel_count() {
        let mut img = SilvestreImage::zeroed(2, 2, ColorSpace::Rgba);
        assert!(matches!(
            img.set_pixel(0, 0, &[0, 0, 0]),
            Err(SilvestreError::ChannelMismatch { expected: 4, got: 3 })
        ));
    }

    #[test]
    fn zeroed_image() {
        let img = SilvestreImage::zeroed(10, 10, ColorSpace::Grayscale);
        assert_eq!(img.pixels().len(), 100);
        assert!(img.pixels().iter().all(|&b| b == 0));
    }

    #[test]
    fn color_space_channels() {
        assert_eq!(ColorSpace::Rgba.channels(), 4);
        assert_eq!(ColorSpace::Rgb.channels(), 3);
        assert_eq!(ColorSpace::Grayscale.channels(), 1);
    }

    #[test]
    fn pixels_mut_allows_direct_modification() {
        let mut img = SilvestreImage::zeroed(1, 1, ColorSpace::Rgba);
        let data = img.pixels_mut();
        data[0] = 255;
        data[1] = 128;
        data[2] = 64;
        data[3] = 32;
        assert_eq!(img.get_pixel(0, 0).unwrap(), &[255, 128, 64, 32]);
    }
}

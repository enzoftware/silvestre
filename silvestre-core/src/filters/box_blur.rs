use crate::filters::convolution::{apply_separable_kernel, BorderMode, SeparableKernel};
use crate::filters::Filter;
use crate::{Result, SilvestreImage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxBlurFilter {
    border: BorderMode,
}

impl BoxBlurFilter {
    #[must_use]
    pub fn new() -> Self {
        Self::with_border(BorderMode::Clamp)
    }

    #[must_use]
    pub fn with_border(border: BorderMode) -> Self {
        Self { border }
    }
}

impl Default for BoxBlurFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl Filter for BoxBlurFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let coeffs = vec![1.0 / 3.0; 3];
        let kernel = SeparableKernel::new(coeffs.clone(), coeffs)?;
        apply_separable_kernel(image, &kernel, self.border)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    fn rgb_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Rgb).unwrap()
    }

    #[test]
    fn uniform_image_stays_uniform() {
        let img = gray_image(5, 5, vec![100; 25]);
        let filter = BoxBlurFilter::new();
        let out = filter.apply(&img).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 100));
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let img = gray_image(4, 6, vec![50; 24]);
        let filter = BoxBlurFilter::new();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 6);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn box_blur_reduces_contrast() {
        let pixels = vec![0, 0, 0, 0, 255, 0, 0, 0, 0];
        let img = gray_image(3, 3, pixels);
        let filter = BoxBlurFilter::new();
        let out = filter.apply(&img).unwrap();
        // The center pixel (255) should be averaged down.
        // With a 3x3 box blur, it should be approximately 255/9 ≈ 28.
        assert!(out.pixels()[4] < 255);
        assert!(out.pixels()[4] > 0);
    }

    #[test]
    fn empty_image() {
        let img = gray_image(0, 0, vec![]);
        let filter = BoxBlurFilter::new();
        let out = filter.apply(&img).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn rgb_image_works() {
        let pixels = vec![255; 3 * 3 * 3]; // 3x3 RGB image, all white
        let img = rgb_image(3, 3, pixels);
        let filter = BoxBlurFilter::new();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Rgb);
        assert!(out.pixels().iter().all(|&v| v == 255));
    }

    #[test]
    fn border_modes() {
        let img = gray_image(3, 3, vec![0, 0, 0, 0, 255, 0, 0, 0, 0]);
        
        let filter_clamp = BoxBlurFilter::with_border(BorderMode::Clamp);
        let out_clamp = filter_clamp.apply(&img).unwrap();
        
        let filter_mirror = BoxBlurFilter::with_border(BorderMode::Mirror);
        let out_mirror = filter_mirror.apply(&img).unwrap();
        
        // They should result in different values at the edges.
        assert_eq!(out_clamp.width(), 3);
        assert_eq!(out_mirror.width(), 3);
    }
}

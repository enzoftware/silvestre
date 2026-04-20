//! Gaussian blur filter.
//!
//! Applies a 2D Gaussian blur using a separable kernel for efficiency.
//! The kernel size is automatically derived from sigma as `2 * ceil(3 * sigma) + 1`.

use crate::filters::convolution::{apply_separable_kernel, BorderMode, SeparableKernel};
use crate::filters::Filter;
use crate::{Result, SilvestreError, SilvestreImage};

/// Gaussian blur filter with configurable sigma and kernel radius.
///
/// The filter uses a separable kernel (two 1D passes) for `O(N * k)` cost
/// instead of `O(N * k²)` for a full 2D convolution.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{Filter, GaussianFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale)?;
/// let blur = GaussianFilter::new(1.0)?;
/// let out = blur.apply(&img)?;
/// assert_eq!(out.width(), 5);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone)]
pub struct GaussianFilter {
    sigma: f32,
    kernel: SeparableKernel,
    border: BorderMode,
}

impl GaussianFilter {
    /// Create a Gaussian filter with the given sigma.
    ///
    /// The kernel radius is automatically chosen as `ceil(3 * sigma)`,
    /// giving a kernel size of `2 * radius + 1`. Uses [`BorderMode::Clamp`].
    ///
    /// Returns an error if `sigma` is not positive or not finite.
    pub fn new(sigma: f32) -> Result<Self> {
        Self::with_border(sigma, BorderMode::Clamp)
    }

    /// Create a Gaussian filter with explicit border mode.
    pub fn with_border(sigma: f32, border: BorderMode) -> Result<Self> {
        if !sigma.is_finite() || sigma <= 0.0 {
            return Err(SilvestreError::InvalidParameter(
                "Gaussian sigma must be a positive finite number".into(),
            ));
        }
        let radius = (3.0 * sigma).ceil() as usize;
        let size = 2 * radius + 1;
        let coeffs = gaussian_1d(sigma, size);
        let kernel = SeparableKernel::new(coeffs.clone(), coeffs)?;
        Ok(Self {
            sigma,
            kernel,
            border,
        })
    }

    /// The standard deviation of the Gaussian.
    #[must_use]
    pub fn sigma(&self) -> f32 {
        self.sigma
    }
}

impl Filter for GaussianFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        apply_separable_kernel(image, &self.kernel, self.border)
    }
}

/// Build a normalized 1D Gaussian kernel of the given odd `size`.
fn gaussian_1d(sigma: f32, size: usize) -> Vec<f32> {
    debug_assert!(size % 2 == 1);
    let half = (size / 2) as f32;
    let two_sigma_sq = 2.0 * sigma * sigma;
    let mut coeffs: Vec<f32> = (0..size)
        .map(|i| {
            let x = i as f32 - half;
            (-x * x / two_sigma_sq).exp()
        })
        .collect();
    let sum: f32 = coeffs.iter().sum();
    for c in &mut coeffs {
        *c /= sum;
    }
    coeffs
}

/// Apply a Gaussian blur to `image` at the given `sigma`, returning the
/// blurred image. This is a convenience used internally by the Canny
/// pipeline and is equivalent to `GaussianFilter::new(sigma)?.apply(image)`.
pub(crate) fn gaussian_blur(image: &SilvestreImage, sigma: f32) -> Result<SilvestreImage> {
    GaussianFilter::new(sigma)?.apply(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn rejects_zero_sigma() {
        assert!(matches!(
            GaussianFilter::new(0.0),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn rejects_negative_sigma() {
        assert!(matches!(
            GaussianFilter::new(-1.0),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn rejects_nan_sigma() {
        assert!(matches!(
            GaussianFilter::new(f32::NAN),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn rejects_infinite_sigma() {
        assert!(matches!(
            GaussianFilter::new(f32::INFINITY),
            Err(SilvestreError::InvalidParameter(_))
        ));
    }

    #[test]
    fn uniform_image_stays_uniform() {
        let img = gray_image(5, 5, vec![100; 25]);
        let filter = GaussianFilter::new(1.0).unwrap();
        let out = filter.apply(&img).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 100));
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let img = gray_image(4, 6, vec![50; 24]);
        let filter = GaussianFilter::new(1.5).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 6);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn blur_reduces_contrast() {
        // Checkerboard-like pattern; blur should reduce the range.
        let img = gray_image(3, 3, vec![0, 255, 0, 255, 0, 255, 0, 255, 0]);
        let filter = GaussianFilter::new(1.0).unwrap();
        let out = filter.apply(&img).unwrap();
        let min = *out.pixels().iter().min().unwrap();
        let max = *out.pixels().iter().max().unwrap();
        assert!(max - min < 255, "blur should reduce contrast");
    }

    #[test]
    fn empty_image() {
        let img = gray_image(0, 0, vec![]);
        let filter = GaussianFilter::new(1.0).unwrap();
        let out = filter.apply(&img).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn sigma_accessor() {
        let filter = GaussianFilter::new(2.5).unwrap();
        assert!((filter.sigma() - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn gaussian_1d_sums_to_one() {
        let coeffs = gaussian_1d(1.0, 7);
        let sum: f32 = coeffs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gaussian_1d_is_symmetric() {
        let coeffs = gaussian_1d(2.0, 9);
        for i in 0..coeffs.len() / 2 {
            assert!(
                (coeffs[i] - coeffs[coeffs.len() - 1 - i]).abs() < 1e-6,
                "coefficient {i} should mirror"
            );
        }
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(GaussianFilter::new(1.0).unwrap());
        let img = gray_image(3, 3, vec![50; 9]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 3);
    }

    #[test]
    fn separable_matches_full_2d_convolution() {
        // Build a 1D Gaussian and its outer product (full 2D kernel), then verify
        // that the separable two-pass result matches the full 2D convolution
        // result within rounding tolerance.
        use crate::filters::convolution::{apply_kernel, Kernel};

        let sigma = 1.0_f32;
        let size = 7_usize; // 2 * ceil(3 * 1.0) + 1 = 7
        let coeffs = gaussian_1d(sigma, size);

        // Build 2D kernel as outer product of the 1D coefficients.
        let mut values_2d = vec![0.0_f32; size * size];
        for (row, &vy) in coeffs.iter().enumerate() {
            for (col, &vx) in coeffs.iter().enumerate() {
                values_2d[row * size + col] = vy * vx;
            }
        }
        let kernel_2d = Kernel::square(values_2d, size).unwrap();
        let sep_kernel =
            SeparableKernel::new(coeffs.clone(), coeffs).unwrap();

        let img = gray_image(8, 8, (0..64u8).collect());

        let full_out = apply_kernel(&img, &kernel_2d, BorderMode::Clamp).unwrap();
        let sep_out = apply_separable_kernel(&img, &sep_kernel, BorderMode::Clamp).unwrap();

        for (i, (a, b)) in full_out
            .pixels()
            .iter()
            .zip(sep_out.pixels().iter())
            .enumerate()
        {
            assert!(
                (i32::from(*a) - i32::from(*b)).abs() <= 1,
                "pixel {i}: full={a} sep={b}: separable and full 2D results diverge"
            );
        }
    }
}

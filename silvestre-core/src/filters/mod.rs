//! Image filters and convolution infrastructure.
//!
//! All image processing operations implement the [`Filter`] trait so they can
//! be composed uniformly. Spatial filters that operate via convolution share
//! the helpers in [`convolution`].

pub mod box_blur;
pub mod canny;
pub mod convolution;
pub mod gaussian;
pub mod median;
pub mod sharpen;
pub mod sobel;

pub use box_blur::BoxBlurFilter;
pub use canny::CannyFilter;
pub use convolution::{apply_kernel, apply_separable_kernel, BorderMode, Kernel, SeparableKernel};
pub use gaussian::GaussianFilter;
pub use median::MedianFilter;
pub use sharpen::SharpenFilter;
pub use sobel::SobelFilter;

use crate::{Result, SilvestreImage};

/// Trait implemented by all image processing filters.
///
/// A filter takes an immutable reference to a [`SilvestreImage`] and produces
/// a new image, leaving the original untouched. This makes filters trivially
/// composable: `gaussian.apply(&sobel.apply(&img)?)?`.
///
/// # Examples
///
/// ```
/// use silvestre_core::effects::InvertFilter;
/// use silvestre_core::{ColorSpace, Filter, SilvestreImage};
///
/// let image = SilvestreImage::new(vec![10, 20, 30], 1, 1, ColorSpace::Rgb)?;
/// let inverted = InvertFilter.apply(&image)?;
///
/// assert_eq!(inverted.pixels(), &[245, 235, 225]);
/// // The original is untouched.
/// assert_eq!(image.pixels(), &[10, 20, 30]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
pub trait Filter: Send + Sync {
    /// Apply this filter to the given image, returning a new image.
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage>;
}

//! Image filters and convolution infrastructure.
//!
//! All image processing operations implement the [`Filter`] trait so they can
//! be composed uniformly. Spatial filters that operate via convolution share
//! the helpers in [`convolution`].

pub mod canny;
pub mod convolution;
pub mod gaussian;
pub mod median;
pub mod sobel;

pub use canny::CannyFilter;
pub use convolution::{apply_kernel, apply_separable_kernel, BorderMode, Kernel, SeparableKernel};
pub use gaussian::GaussianFilter;
pub use median::MedianFilter;
pub use sobel::SobelFilter;

use crate::{Result, SilvestreImage};

/// Trait implemented by all image processing filters.
///
/// A filter takes an immutable reference to a [`SilvestreImage`] and produces
/// a new image, leaving the original untouched. This makes filters trivially
/// composable: `gaussian.apply(&sobel.apply(&img)?)?`.
pub trait Filter: Send + Sync {
    /// Apply this filter to the given image, returning a new image.
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage>;
}

pub mod analysis;
pub mod effects;
pub mod filters;
pub mod transform;

mod error;
mod image;
mod io;

pub use error::SilvestreError;
pub use image::{ColorSpace, SilvestreImage};
pub use io::ImageFormat;

/// Result type alias for silvestre operations.
pub type Result<T> = std::result::Result<T, SilvestreError>;

/// Trait implemented by all image processing filters.
pub trait Filter {
    /// Apply this filter to the given image, returning a new image.
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage>;
}

//! # silvestre-core
//!
//! The portable image-processing engine at the heart of silvestre. It has no
//! platform dependencies, so the same filters run natively, on the CLI, in the
//! browser via WebAssembly, and on mobile through the FFI and Flutter bindings.
//!
//! ## Core concepts
//!
//! - [`SilvestreImage`] — an owned pixel buffer plus its dimensions and
//!   [`ColorSpace`]. [`Filter`] operations take an image by reference and
//!   return a new one, leaving the input untouched. (Low-level accessors such
//!   as [`SilvestreImage::set_pixel`] and [`SilvestreImage::pixels_mut`] edit
//!   an image in place.)
//! - [`Filter`] — the common trait implemented by every operation. Filters are
//!   [`Send`] + [`Sync`], so they can be shared across threads and composed
//!   freely.
//! - [`SilvestreError`] / [`Result`] — the error type and result alias returned
//!   by all fallible operations.
//!
//! Operations are grouped into modules: [`effects`] (per-pixel color effects),
//! [`filters`] (convolution and spatial filters), [`transform`] (geometric
//! transforms), and [`analysis`] (histograms).
//!
//! ## Example
//!
//! Load an image conceptually, then convert it to grayscale and adjust
//! brightness by applying filters in sequence:
//!
//! ```
//! use silvestre_core::effects::{BrightnessFilter, GrayscaleFilter};
//! use silvestre_core::{ColorSpace, Filter, SilvestreImage};
//!
//! // A 2x2 RGBA image (here constructed in memory; usually loaded from disk).
//! let image = SilvestreImage::new(vec![128; 2 * 2 * 4], 2, 2, ColorSpace::Rgba)?;
//!
//! let gray = GrayscaleFilter.apply(&image)?;
//! let brighter = BrightnessFilter::new(40).apply(&gray)?;
//!
//! assert_eq!(brighter.color_space(), ColorSpace::Grayscale);
//! # Ok::<_, silvestre_core::SilvestreError>(())
//! ```
#![warn(missing_docs)]

pub mod analysis;
pub mod effects;
pub mod filters;
pub mod simd;
pub mod transform;

mod error;
mod image;
mod io;

pub use error::SilvestreError;
pub use filters::Filter;
pub use image::{ColorSpace, SilvestreImage};
pub use io::ImageFormat;

/// Result type alias for silvestre operations.
///
/// Shorthand for `std::result::Result<T, SilvestreError>` returned by every
/// fallible operation in this crate.
pub type Result<T> = std::result::Result<T, SilvestreError>;

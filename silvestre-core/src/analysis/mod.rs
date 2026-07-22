//! Image analysis tools.
//!
//! Currently provides per-channel [`Histogram`] computation for inspecting the
//! tonal distribution of an image.

pub mod histogram;

pub use histogram::Histogram;

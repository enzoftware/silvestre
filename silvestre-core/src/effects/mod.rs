//! Color and pixel-level effects.
//!
//! These filters operate on each pixel independently (no spatial neighborhood):
//! [`BrightnessFilter`], [`ContrastFilter`], [`GrayscaleFilter`],
//! [`InvertFilter`], and [`SepiaFilter`].

pub mod brightness;
pub mod contrast;
pub mod grayscale;
pub mod invert;
pub mod sepia;

pub use brightness::BrightnessFilter;
pub use contrast::ContrastFilter;
pub use grayscale::GrayscaleFilter;
pub use invert::InvertFilter;
pub use sepia::SepiaFilter;

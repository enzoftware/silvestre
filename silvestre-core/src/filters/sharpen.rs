//! Sharpen filter.

use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Sharpen filter.
#[derive(Debug, Clone, Default)]
pub struct SharpenFilter;

impl SharpenFilter {
    /// Create a new sharpen filter.
    pub fn new() -> Self {
        Self
    }
}

impl Filter for SharpenFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        Ok(image.clone())
    }
}

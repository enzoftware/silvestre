//! Box blur filter.

use crate::filters::Filter;
use crate::{Result, SilvestreImage};

/// Box blur filter.
#[derive(Debug, Clone, Default)]
pub struct BoxBlurFilter;

impl BoxBlurFilter {
    /// Create a new box blur filter.
    pub fn new() -> Self {
        Self
    }
}

impl Filter for BoxBlurFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        Ok(image.clone())
    }
}

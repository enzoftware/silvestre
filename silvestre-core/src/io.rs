use std::path::Path;

use image::GenericImageView;

use crate::{ColorSpace, Result, SilvestreImage};

impl SilvestreImage {
    /// Load an image from a file path. The image is converted to RGBA.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();
        Ok(Self::new(rgba.into_raw(), width, height, ColorSpace::Rgba)?)
    }

    /// Save the image to a file path. The format is inferred from the extension.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let color_type = match self.color_space() {
            ColorSpace::Rgba => image::ColorType::Rgba8,
            ColorSpace::Rgb => image::ColorType::Rgb8,
            ColorSpace::Grayscale => image::ColorType::L8,
        };
        image::save_buffer(path, self.pixels(), self.width(), self.height(), color_type)?;
        Ok(())
    }
}

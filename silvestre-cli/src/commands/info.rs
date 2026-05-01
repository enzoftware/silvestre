use crate::Result;
use clap::Parser;
use image::{GenericImageView, ImageReader};
use silvestre_core::{analysis::histogram::Histogram, SilvestreImage};
use std::path::PathBuf;

#[derive(Parser)]
pub struct InfoCommand {
    /// Path to the image file
    #[arg(long, short)]
    input: PathBuf,
}

impl InfoCommand {
    pub fn execute(self) -> Result<()> {
        if !self.input.exists() {
            return Err(crate::CliError::Custom(format!(
                "Input file not found: {}",
                self.input.display()
            )));
        }

        // Load image using image crate
        let img = ImageReader::open(&self.input)?
            .with_guessed_format()?
            .decode()?;

        let (width, height) = img.dimensions();
        let color_type = img.color();

        println!("Image Information:");
        println!("  File: {}", self.input.display());
        println!("  Dimensions: {}x{}", width, height);
        println!("  Format: {:?}", color_type);
        println!("  Color Type: {}", format_color_type(color_type));
        println!();

        // Try to convert to silvestre format and compute histogram
        match convert_to_silvestre(&img) {
            Ok(silvestre_img) => {
                let histogram = Histogram::compute(&silvestre_img);

                println!("Statistics:");
                for i in 0..histogram.num_channels() {
                    let stat = histogram.stats(i);
                    println!("  Channel {}: min={}, max={}, mean={:.2}, stddev={:.2}",
                             i, stat.min, stat.max, stat.mean, stat.std_dev);
                }
            }
            Err(_) => {
                println!("Note: Could not compute detailed statistics for this image format.");
            }
        }

        Ok(())
    }
}

fn format_color_type(color_type: image::ColorType) -> String {
    match color_type {
        image::ColorType::L8 => "Grayscale (8-bit)".to_string(),
        image::ColorType::La8 => "Grayscale with Alpha (8-bit)".to_string(),
        image::ColorType::Rgb8 => "RGB (8-bit)".to_string(),
        image::ColorType::Rgba8 => "RGBA (8-bit)".to_string(),
        _ => format!("{:?}", color_type),
    }
}

fn convert_to_silvestre(
    img: &image::DynamicImage,
) -> Result<SilvestreImage> {
    use silvestre_core::ColorSpace;

    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    let pixels = rgba.to_vec();

    SilvestreImage::new(pixels, width, height, ColorSpace::Rgba)
        .map_err(|e| crate::CliError::Silvestre(e))
}

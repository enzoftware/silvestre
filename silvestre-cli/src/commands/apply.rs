use crate::{CliError, Result};
use clap::Parser;
use image::{GenericImageView, ImageReader};
use indicatif::ProgressBar;
use silvestre_core::{
    effects::{BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter},
    filters::Filter,
    transform::{CropFilter, Interpolation, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter},
    ColorSpace, SilvestreImage,
};
use std::path::PathBuf;

#[derive(Parser)]
pub struct ApplyCommand {
    /// Filter name (brightness, contrast, grayscale, invert, box-blur, gaussian, median, crop, mirror, resize, rotate, etc.)
    filter: String,

    /// Input image path
    #[arg(long, short)]
    input: PathBuf,

    /// Output image path
    #[arg(long, short)]
    output: PathBuf,

    /// Filter parameters (format: key=value)
    #[arg(long)]
    params: Vec<String>,
}

impl ApplyCommand {
    pub fn execute(self) -> Result<()> {
        if !self.input.exists() {
            return Err(CliError::Custom(format!(
                "Input file not found: {}",
                self.input.display()
            )));
        }

        let pb = ProgressBar::new_spinner();
        pb.set_message("Loading image...");

        // Load image
        let img = ImageReader::open(&self.input)?
            .with_guessed_format()?
            .decode()?;

        let silvestre_img = convert_to_silvestre(&img)?;

        pb.set_message(format!("Applying filter: {}...", self.filter));

        // Parse parameters
        let params = parse_params(&self.params)?;

        // Apply filter
        let result = match self.filter.as_str() {
            "brightness" => apply_brightness(&silvestre_img, &params)?,
            "contrast" => apply_contrast(&silvestre_img, &params)?,
            "grayscale" => {
                GrayscaleFilter.apply(&silvestre_img)?
            }
            "invert" => {
                InvertFilter.apply(&silvestre_img)?
            }
            "crop" => apply_crop(&silvestre_img, &params)?,
            "mirror" => apply_mirror(&silvestre_img, &params)?,
            "resize" => apply_resize(&silvestre_img, &params)?,
            "rotate" => apply_rotate(&silvestre_img, &params)?,
            _ => {
                return Err(CliError::Custom(format!(
                    "Unknown filter: {}. Run 'silvestre list' to see available filters.",
                    self.filter
                )))
            }
        };

        pb.set_message("Saving image...");

        // Save result
        let output_img = convert_from_silvestre(&result)?;
        output_img.save(&self.output)?;

        pb.finish_with_message(format!("✓ Saved to {}", self.output.display()));

        Ok(())
    }
}

fn parse_params(params: &[String]) -> Result<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();
    for param in params {
        let parts: Vec<&str> = param.split('=').collect();
        if parts.len() != 2 {
            return Err(CliError::Custom(
                "Invalid parameter format. Use key=value".to_string(),
            ));
        }
        map.insert(parts[0].to_string(), parts[1].to_string());
    }
    Ok(map)
}

fn apply_brightness(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let delta = params
        .get("delta")
        .ok_or_else(|| CliError::Custom("brightness requires --params delta=<value>".to_string()))?
        .parse::<i32>()
        .map_err(|_| CliError::Custom("delta must be an integer".to_string()))?;

    let filter = BrightnessFilter::new(delta);
    Ok(filter.apply(img)?)
}

fn apply_contrast(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let factor = params
        .get("factor")
        .ok_or_else(|| CliError::Custom("contrast requires --params factor=<value>".to_string()))?
        .parse::<f32>()
        .map_err(|_| CliError::Custom("factor must be a number".to_string()))?;

    let filter = ContrastFilter::new(factor)?;
    Ok(filter.apply(img)?)
}

fn apply_crop(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let x = parse_u32(params, "x")?;
    let y = parse_u32(params, "y")?;
    let width = parse_u32(params, "width")?;
    let height = parse_u32(params, "height")?;

    let filter = CropFilter::new(x, y, width, height);
    Ok(filter.apply(img)?)
}

fn apply_mirror(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let mode_str = params
        .get("mode")
        .ok_or_else(|| CliError::Custom("mirror requires --params mode=<h|v|both>".to_string()))?
        .as_str();

    let mode = match mode_str {
        "h" | "horizontal" => MirrorMode::Horizontal,
        "v" | "vertical" => MirrorMode::Vertical,
        "both" => MirrorMode::Both,
        _ => return Err(CliError::Custom("mode must be h, v, or both".to_string())),
    };

    let filter = MirrorFilter::new(mode);
    Ok(filter.apply(img)?)
}

fn apply_resize(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let width = parse_u32(params, "width")?;
    let height = parse_u32(params, "height")?;
    let interpolation = params.get("interpolation").map(|s| s.as_str()).unwrap_or("bilinear");

    let interp = match interpolation {
        "nearest" => Interpolation::NearestNeighbor,
        "bilinear" => Interpolation::Bilinear,
        _ => return Err(CliError::Custom("interpolation must be nearest or bilinear".to_string())),
    };

    let filter = ResizeFilter::new(width, height, interp);
    Ok(filter.apply(img)?)
}

fn apply_rotate(
    img: &SilvestreImage,
    params: &std::collections::HashMap<String, String>,
) -> Result<SilvestreImage> {
    let angle = params
        .get("angle")
        .ok_or_else(|| CliError::Custom("rotate requires --params angle=<degrees>".to_string()))?
        .parse::<f64>()
        .map_err(|_| CliError::Custom("angle must be a number".to_string()))?;

    let filter = RotateFilter::new(angle, 0, [0, 0, 0]);
    Ok(filter.apply(img)?)
}

fn parse_u32(params: &std::collections::HashMap<String, String>, key: &str) -> Result<u32> {
    params
        .get(key)
        .ok_or_else(|| CliError::Custom(format!("missing parameter: {}", key)))?
        .parse::<u32>()
        .map_err(|_| CliError::Custom(format!("{} must be an unsigned integer", key)))
}

fn convert_to_silvestre(img: &image::DynamicImage) -> Result<SilvestreImage> {
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    let pixels = rgba.to_vec();

    SilvestreImage::new(pixels, width, height, ColorSpace::Rgba)
        .map_err(|e| CliError::Silvestre(e))
}

fn convert_from_silvestre(silvestre_img: &SilvestreImage) -> Result<image::DynamicImage> {
    if silvestre_img.width() == 0 || silvestre_img.height() == 0 {
        return Err(CliError::Custom("Cannot save empty image".to_string()));
    }

    let img_buffer = image::RgbaImage::from_raw(
        silvestre_img.width(),
        silvestre_img.height(),
        silvestre_img.pixels().to_vec(),
    )
    .ok_or_else(|| {
        CliError::Custom("Failed to create image from pixel buffer".to_string())
    })?;

    Ok(image::DynamicImage::ImageRgba8(img_buffer))
}

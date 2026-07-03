//! Shared filter application logic.
//!
//! These helpers map a filter name plus a parameter string to a concrete
//! `silvestre_core` filter and apply it. They are used both by the single
//! "Apply Filter" screen and by the multi-step "Pipeline" screen, so the
//! parsing and error-reporting rules stay identical between the two.

use silvestre_core::effects::{BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter};
use silvestre_core::filters::Filter;
use silvestre_core::transform::{
    CropFilter, Interpolation, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter,
};
use silvestre_core::{ColorSpace, SilvestreImage};

/// Canonical list of filter names the CLI knows how to build, together with a
/// short human-readable hint describing the parameters they expect.
pub const KNOWN_FILTERS: &[(&str, &str)] = &[
    ("brightness", "delta (-255 to 255)"),
    ("contrast", "factor (0.0+)"),
    ("grayscale", "no parameters"),
    ("invert", "no parameters"),
    ("crop", "x,y,width,height"),
    ("mirror", "h | v | both"),
    ("resize", "width,height"),
    ("rotate", "angle in degrees"),
];

/// Returns `true` if `name` is a filter the CLI can apply.
pub fn is_known_filter(name: &str) -> bool {
    KNOWN_FILTERS.iter().any(|(n, _)| *n == name)
}

/// Validate that `filter_name` is known and that `params` parses correctly,
/// without touching any image data.
///
/// This lets a whole pipeline be checked up front, before any expensive image
/// work happens, so a typo in step 3 fails before step 1 runs.
pub fn validate_filter(filter_name: &str, params: &str) -> Result<(), String> {
    match filter_name {
        "brightness" => {
            parse_i32(params, "Brightness requires a number (-255 to 255)")?;
            Ok(())
        }
        "contrast" => {
            let factor = parse_f32(params, "Contrast requires a decimal number")?;
            // Surface the same error the core constructor would, up front.
            ContrastFilter::new(factor).map_err(|e| e.to_string())?;
            Ok(())
        }
        "grayscale" | "invert" => Ok(()),
        "crop" => {
            parse_crop(params)?;
            Ok(())
        }
        "mirror" => {
            parse_mirror_mode(params)?;
            Ok(())
        }
        "resize" => {
            parse_resize(params)?;
            Ok(())
        }
        "rotate" => {
            parse_finite_f64(params, "Rotate requires a finite angle in degrees")?;
            Ok(())
        }
        _ => Err(format!("Unknown filter: {}", filter_name)),
    }
}

/// Apply a single filter identified by `filter_name` to `img`, using the raw
/// `params` string as its arguments.
pub fn apply_named_filter(
    img: &SilvestreImage,
    filter_name: &str,
    params: &str,
) -> Result<SilvestreImage, String> {
    match filter_name {
        "brightness" => {
            let delta = parse_i32(params, "Brightness requires a number (-255 to 255)")?;
            BrightnessFilter::new(delta)
                .apply(img)
                .map_err(|e| format!("Brightness filter error: {}", e))
        }
        "contrast" => {
            let factor = parse_f32(params, "Contrast requires a decimal number")?;
            let filter = ContrastFilter::new(factor)
                .map_err(|e| format!("Contrast filter error: {}", e))?;
            filter
                .apply(img)
                .map_err(|e| format!("Contrast filter error: {}", e))
        }
        "grayscale" => GrayscaleFilter
            .apply(img)
            .map_err(|e| format!("Grayscale filter error: {}", e)),
        "invert" => InvertFilter
            .apply(img)
            .map_err(|e| format!("Invert filter error: {}", e)),
        "crop" => {
            let (x, y, w, h) = parse_crop(params)?;
            CropFilter::new(x, y, w, h)
                .apply(img)
                .map_err(|e| format!("Crop filter error: {}", e))
        }
        "mirror" => {
            let mode = parse_mirror_mode(params)?;
            MirrorFilter::new(mode)
                .apply(img)
                .map_err(|e| format!("Mirror filter error: {}", e))
        }
        "resize" => {
            let (w, h) = parse_resize(params)?;
            ResizeFilter::new(w, h, Interpolation::Bilinear)
                .apply(img)
                .map_err(|e| format!("Resize filter error: {}", e))
        }
        "rotate" => {
            let angle = parse_finite_f64(params, "Rotate requires a finite angle in degrees")?;
            RotateFilter::new(angle, 0, [0, 0, 0])
                .apply(img)
                .map_err(|e| format!("Rotate filter error: {}", e))
        }
        _ => Err(format!("Unknown filter: {}", filter_name)),
    }
}

/// Convert a `SilvestreImage` into a `DynamicImage` for saving, honoring its
/// color space.
///
/// Filters such as grayscale change the channel count (and thus the buffer
/// length), so the buffer can't be assumed to be RGBA. This picks the matching
/// `image` buffer type based on the actual color space.
pub fn silvestre_to_dynamic(img: &SilvestreImage) -> Result<image::DynamicImage, String> {
    if img.width() == 0 || img.height() == 0 {
        return Err("Cannot save an empty image".to_string());
    }
    let (w, h) = (img.width(), img.height());
    let pixels = img.pixels().to_vec();

    let dynamic = match img.color_space() {
        ColorSpace::Rgba => image::RgbaImage::from_raw(w, h, pixels)
            .map(image::DynamicImage::ImageRgba8),
        ColorSpace::Rgb => {
            image::RgbImage::from_raw(w, h, pixels).map(image::DynamicImage::ImageRgb8)
        }
        ColorSpace::Grayscale => {
            image::GrayImage::from_raw(w, h, pixels).map(image::DynamicImage::ImageLuma8)
        }
    };

    dynamic.ok_or_else(|| "Failed to create image buffer from pixels".to_string())
}

fn parse_i32(params: &str, msg: &str) -> Result<i32, String> {
    params.trim().parse::<i32>().map_err(|_| msg.to_string())
}

fn parse_f32(params: &str, msg: &str) -> Result<f32, String> {
    params.trim().parse::<f32>().map_err(|_| msg.to_string())
}

/// Parse an `f64` and reject non-finite values (NaN, ±infinity), which would
/// otherwise flow into infallible filter constructors and produce garbage.
fn parse_finite_f64(params: &str, msg: &str) -> Result<f64, String> {
    let value = params.trim().parse::<f64>().map_err(|_| msg.to_string())?;
    if !value.is_finite() {
        return Err(msg.to_string());
    }
    Ok(value)
}

fn parse_crop(params: &str) -> Result<(u32, u32, u32, u32), String> {
    let parts: Vec<&str> = params.trim().split(',').collect();
    if parts.len() != 4 {
        return Err("Crop requires 4 params: x, y, width, height".to_string());
    }
    let x = parts[0]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid crop x coordinate".to_string())?;
    let y = parts[1]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid crop y coordinate".to_string())?;
    let w = parts[2]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid crop width".to_string())?;
    let h = parts[3]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid crop height".to_string())?;
    Ok((x, y, w, h))
}

fn parse_resize(params: &str) -> Result<(u32, u32), String> {
    let parts: Vec<&str> = params.trim().split(',').collect();
    if parts.len() != 2 {
        return Err("Resize requires 2 params: width, height".to_string());
    }
    let w = parts[0]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid resize width".to_string())?;
    let h = parts[1]
        .trim()
        .parse::<u32>()
        .map_err(|_| "Invalid resize height".to_string())?;
    Ok((w, h))
}

fn parse_mirror_mode(params: &str) -> Result<MirrorMode, String> {
    match params.trim().to_lowercase().as_str() {
        "h" | "horizontal" => Ok(MirrorMode::Horizontal),
        "v" | "vertical" => Ok(MirrorMode::Vertical),
        "both" => Ok(MirrorMode::Both),
        _ => Err("Mirror mode should be: h, v, or both".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use silvestre_core::ColorSpace;

    /// A small solid-color RGBA test image.
    fn test_image(w: u32, h: u32) -> SilvestreImage {
        let pixels = vec![128u8; (w * h * 4) as usize];
        SilvestreImage::new(pixels, w, h, ColorSpace::Rgba).unwrap()
    }

    #[test]
    fn known_filters_are_recognized() {
        assert!(is_known_filter("grayscale"));
        assert!(is_known_filter("resize"));
        assert!(!is_known_filter("nope"));
        assert!(!is_known_filter(""));
    }

    #[test]
    fn validate_accepts_valid_params() {
        assert!(validate_filter("brightness", "50").is_ok());
        assert!(validate_filter("contrast", "1.5").is_ok());
        assert!(validate_filter("grayscale", "").is_ok());
        assert!(validate_filter("invert", "ignored").is_ok());
        assert!(validate_filter("crop", "0,0,10,10").is_ok());
        assert!(validate_filter("mirror", "both").is_ok());
        assert!(validate_filter("resize", "20,20").is_ok());
        assert!(validate_filter("rotate", "90").is_ok());
    }

    #[test]
    fn validate_rejects_bad_params() {
        assert!(validate_filter("brightness", "abc").is_err());
        assert!(validate_filter("contrast", "notnum").is_err());
        assert!(validate_filter("crop", "1,2,3").is_err());
        assert!(validate_filter("resize", "10").is_err());
        assert!(validate_filter("mirror", "sideways").is_err());
        assert!(validate_filter("rotate", "").is_err());
    }

    #[test]
    fn validate_rejects_non_finite_rotate() {
        assert!(validate_filter("rotate", "nan").is_err());
        assert!(validate_filter("rotate", "inf").is_err());
        assert!(validate_filter("rotate", "-inf").is_err());
    }

    #[test]
    fn apply_rejects_non_finite_rotate() {
        let img = test_image(4, 4);
        assert!(apply_named_filter(&img, "rotate", "inf").is_err());
    }

    #[test]
    fn validate_rejects_unknown_filter() {
        let err = validate_filter("blur9000", "1").unwrap_err();
        assert!(err.contains("Unknown filter"));
    }

    #[test]
    fn apply_grayscale_preserves_dimensions() {
        let img = test_image(4, 3);
        let out = apply_named_filter(&img, "grayscale", "").unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 3);
    }

    #[test]
    fn apply_resize_changes_dimensions() {
        let img = test_image(4, 4);
        let out = apply_named_filter(&img, "resize", "8,6").unwrap();
        assert_eq!(out.width(), 8);
        assert_eq!(out.height(), 6);
    }

    #[test]
    fn apply_crop_produces_subregion() {
        let img = test_image(10, 10);
        let out = apply_named_filter(&img, "crop", "0,0,4,5").unwrap();
        assert_eq!(out.width(), 4);
        assert_eq!(out.height(), 5);
    }

    #[test]
    fn apply_unknown_filter_errors() {
        let img = test_image(2, 2);
        assert!(apply_named_filter(&img, "mystery", "").is_err());
    }

    #[test]
    fn apply_bad_params_errors() {
        let img = test_image(2, 2);
        let err = apply_named_filter(&img, "brightness", "xyz").unwrap_err();
        assert!(err.contains("Brightness"));
    }
}

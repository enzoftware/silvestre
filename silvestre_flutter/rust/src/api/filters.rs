use silvestre_core::effects::{
    BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter, SepiaFilter,
};
use silvestre_core::filters::{
    BoxBlurFilter, CannyFilter, GaussianFilter, MedianFilter, SharpenFilter, SobelFilter,
};
use silvestre_core::transform::{CropFilter, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter};
use silvestre_core::{Filter, SilvestreImage};

use super::image::SilvestreImageWrapper;

/// Apply a named filter to an image, returning a new image.
///
/// # Parameters
///
/// - `name`: one of `grayscale`, `invert`, `sepia`, `brightness`, `contrast`,
///   `sharpen`, `box_blur`, `sobel`, `gaussian`, `median`, `canny`,
///   `crop`, `resize`, `rotate`, `mirror`.
/// - `params_json`: a JSON object with filter-specific parameters.
///   Pass `"{}"` or `""` for filters that take no parameters.
///
/// ## Parameter table
///
/// | Filter       | JSON params                                        |
/// |--------------|----------------------------------------------------|
/// | `grayscale`  | *(none)*                                           |
/// | `invert`     | *(none)*                                           |
/// | `sepia`      | *(none)*                                           |
/// | `sharpen`    | *(none)*                                           |
/// | `box_blur`   | *(none)*                                           |
/// | `sobel`      | *(none)*                                           |
/// | `brightness` | `{"delta": <i32>}`                                 |
/// | `contrast`   | `{"factor": <f32>}`                                |
/// | `gaussian`   | `{"sigma": <f32>}`                                 |
/// | `median`     | `{"size": <usize>}`                                |
/// | `canny`      | `{"low": <f32>, "high": <f32>, "sigma": <f32>}`   |
/// | `crop`       | `{"x": <u32>, "y": <u32>, "w": <u32>, "h": <u32>}`|
/// | `resize`     | `{"w": <u32>, "h": <u32>}`                         |
/// | `rotate`     | `{"angle": <f64>}`                                 |
/// | `mirror`     | `{"mode": "horizontal"\|"vertical"\|"both"}`       |
pub fn apply_filter(
    img: &SilvestreImageWrapper,
    name: String,
    params_json: String,
) -> Result<SilvestreImageWrapper, String> {
    let params: serde_json::Value = if params_json.is_empty() {
        serde_json::Value::Object(serde_json::Map::new())
    } else {
        serde_json::from_str(&params_json)
            .map_err(|e| format!("invalid JSON params: {e}"))?
    };

    let result = apply_named_filter(&name, &img.inner, &params)?;
    Ok(SilvestreImageWrapper { inner: result })
}

// ---------------------------------------------------------------------------
// Internal dispatch
// ---------------------------------------------------------------------------

fn apply_named_filter(
    name: &str,
    image: &SilvestreImage,
    params: &serde_json::Value,
) -> Result<SilvestreImage, String> {
    match name {
        // Effects (no params)
        "grayscale" => GrayscaleFilter.apply(image).map_err(|e| e.to_string()),
        "invert" => InvertFilter.apply(image).map_err(|e| e.to_string()),
        "sepia" => SepiaFilter.apply(image).map_err(|e| e.to_string()),

        // Spatial filters (no params)
        "sharpen" => SharpenFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(|e| e.to_string()),
        "box_blur" => BoxBlurFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(|e| e.to_string()),
        "sobel" => SobelFilter::new().apply(image).map_err(|e| e.to_string()),

        // Effects with params
        "brightness" => {
            let delta = get_i32(params, "delta")?;
            BrightnessFilter::new(delta)
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "contrast" => {
            let factor = get_f32(params, "factor")?;
            ContrastFilter::new(factor)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }

        // Filters with params
        "gaussian" => {
            let sigma = get_f32(params, "sigma")?;
            GaussianFilter::new(sigma)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "median" => {
            let size = get_usize(params, "size")?;
            MedianFilter::new(size)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "canny" => {
            let low = get_f32(params, "low")?;
            let high = get_f32(params, "high")?;
            let sigma = get_f32(params, "sigma")?;
            CannyFilter::new(low, high, sigma)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }

        // Transforms
        "crop" => {
            let x = get_u32(params, "x")?;
            let y = get_u32(params, "y")?;
            let w = get_u32(params, "w")?;
            let h = get_u32(params, "h")?;
            CropFilter::new(x, y, w, h)
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "resize" => {
            let w = get_u32(params, "w")?;
            let h = get_u32(params, "h")?;
            ResizeFilter::new(
                w,
                h,
                silvestre_core::transform::Interpolation::Bilinear,
            )
            .apply(image)
            .map_err(|e| e.to_string())
        }
        "rotate" => {
            let angle = get_f64(params, "angle")?;
            RotateFilter::new(angle, 0, [0, 0, 0])
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "mirror" => {
            let mode_str = get_str(params, "mode")?;
            let mode = match mode_str.as_str() {
                "horizontal" => MirrorMode::Horizontal,
                "vertical" => MirrorMode::Vertical,
                "both" => MirrorMode::Both,
                _ => return Err(format!("unknown mirror mode: {mode_str}")),
            };
            MirrorFilter::new(mode)
                .apply(image)
                .map_err(|e| e.to_string())
        }

        _ => Err(format!("unknown filter: {name}")),
    }
}

// ---------------------------------------------------------------------------
// JSON param helpers
// ---------------------------------------------------------------------------

fn get_f32(params: &serde_json::Value, key: &str) -> Result<f32, String> {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .ok_or_else(|| format!("missing or invalid param: {key}"))
}

fn get_f64(params: &serde_json::Value, key: &str) -> Result<f64, String> {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| format!("missing or invalid param: {key}"))
}

fn get_i32(params: &serde_json::Value, key: &str) -> Result<i32, String> {
    let raw = params
        .get(key)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| format!("missing or invalid param: {key}"))?;
    i32::try_from(raw).map_err(|_| format!("param out of range: {key}"))
}

fn get_u32(params: &serde_json::Value, key: &str) -> Result<u32, String> {
    let raw = params
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("missing or invalid param: {key}"))?;
    u32::try_from(raw).map_err(|_| format!("param out of range: {key}"))
}

fn get_usize(params: &serde_json::Value, key: &str) -> Result<usize, String> {
    let raw = params
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| format!("missing or invalid param: {key}"))?;
    usize::try_from(raw).map_err(|_| format!("param out of range: {key}"))
}

fn get_str(params: &serde_json::Value, key: &str) -> Result<String, String> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("missing or invalid param: {key}"))
}

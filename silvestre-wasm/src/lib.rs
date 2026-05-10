use wasm_bindgen::prelude::*;
use web_sys::ImageData;

use silvestre_core::effects::{
    BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter, SepiaFilter,
};
use silvestre_core::filters::{
    BoxBlurFilter, CannyFilter, GaussianFilter, MedianFilter, SharpenFilter, SobelFilter,
};
use silvestre_core::transform::{CropFilter, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter};
use silvestre_core::{ColorSpace, Filter, ImageFormat, SilvestreImage};

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// WASM wrapper around `SilvestreImage`.
#[wasm_bindgen]
pub struct WasmImage {
    inner: SilvestreImage,
}

#[wasm_bindgen]
impl WasmImage {
    /// Load an image from raw file bytes (PNG, JPEG, BMP).
    #[wasm_bindgen(js_name = "loadFromBytes")]
    pub fn load_from_bytes(data: &[u8]) -> Result<WasmImage, JsValue> {
        let inner = SilvestreImage::load_from_memory(data).map_err(err_to_js)?;
        Ok(WasmImage { inner })
    }

    /// Load an image from an HTML canvas `ImageData` object.
    #[wasm_bindgen(js_name = "loadFromImageData")]
    pub fn load_from_image_data(data: &ImageData) -> Result<WasmImage, JsValue> {
        let width = data.width();
        let height = data.height();
        let pixels = data.data().0;
        let inner =
            SilvestreImage::new(pixels, width, height, ColorSpace::Rgba).map_err(err_to_js)?;
        Ok(WasmImage { inner })
    }

    /// Apply a named filter, returning a new `WasmImage`.
    ///
    /// `params` is a JS object with filter-specific parameters.
    #[wasm_bindgen(js_name = "applyFilter")]
    pub fn apply_filter(&self, name: &str, params: JsValue) -> Result<WasmImage, JsValue> {
        let params_json: serde_json::Value = if params.is_undefined() || params.is_null() {
            serde_json::Value::Object(serde_json::Map::new())
        } else {
            serde_wasm_bindgen::from_value(params).map_err(|e| JsValue::from_str(&e.to_string()))?
        };
        let result = apply_named_filter(name, &self.inner, &params_json)?;
        Ok(WasmImage { inner: result })
    }

    /// Convert the image to an RGBA `ImageData` suitable for canvas rendering.
    #[wasm_bindgen(js_name = "toImageData")]
    pub fn to_image_data(&self) -> Result<ImageData, JsValue> {
        let rgba = to_rgba_pixels(&self.inner);
        ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(rgba.as_slice()),
            self.inner.width(),
            self.inner.height(),
        )
    }

    /// Encode the image to bytes in the given format ("png", "jpeg", "bmp").
    #[wasm_bindgen(js_name = "toBytes")]
    pub fn to_bytes(&self, format: &str) -> Result<Vec<u8>, JsValue> {
        let fmt = parse_format(format)?;
        self.inner.encode_to_memory(fmt).map_err(err_to_js)
    }

    /// Image width in pixels.
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.inner.width()
    }

    /// Image height in pixels.
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.inner.height()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn err_to_js(e: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn parse_format(s: &str) -> Result<ImageFormat, JsValue> {
    match s.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "bmp" => Ok(ImageFormat::Bmp),
        _ => Err(JsValue::from_str(&format!("unsupported format: {s}"))),
    }
}

/// Convert any `SilvestreImage` color space to RGBA pixels.
fn to_rgba_pixels(img: &SilvestreImage) -> Vec<u8> {
    let pixels = img.pixels();
    match img.color_space() {
        ColorSpace::Rgba => pixels.to_vec(),
        ColorSpace::Rgb => {
            let mut rgba = Vec::with_capacity(pixels.len() / 3 * 4);
            for chunk in pixels.chunks_exact(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        ColorSpace::Grayscale => {
            let mut rgba = Vec::with_capacity(pixels.len() * 4);
            for &g in pixels {
                rgba.push(g);
                rgba.push(g);
                rgba.push(g);
                rgba.push(255);
            }
            rgba
        }
    }
}

// ---------------------------------------------------------------------------
// Filter dispatch
// ---------------------------------------------------------------------------

fn get_f32(params: &serde_json::Value, key: &str) -> Result<f32, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn get_f64(params: &serde_json::Value, key: &str) -> Result<f64, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_f64())
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn get_i32(params: &serde_json::Value, key: &str) -> Result<i32, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn get_u32(params: &serde_json::Value, key: &str) -> Result<u32, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn get_usize(params: &serde_json::Value, key: &str) -> Result<usize, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn get_str(params: &serde_json::Value, key: &str) -> Result<String, JsValue> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| JsValue::from_str(&format!("missing or invalid param: {key}")))
}

fn apply_named_filter(
    name: &str,
    image: &SilvestreImage,
    params: &serde_json::Value,
) -> Result<SilvestreImage, JsValue> {
    match name {
        // Effects (no params)
        "grayscale" => GrayscaleFilter.apply(image).map_err(err_to_js),
        "invert" => InvertFilter.apply(image).map_err(err_to_js),
        "sepia" => SepiaFilter.apply(image).map_err(err_to_js),
        "sharpen" => SharpenFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(err_to_js),
        "box_blur" => BoxBlurFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(err_to_js),
        "sobel" => SobelFilter::new().apply(image).map_err(err_to_js),

        // Effects with params
        "brightness" => {
            let delta = get_i32(params, "delta")?;
            BrightnessFilter::new(delta)
                .apply(image)
                .map_err(err_to_js)
        }
        "contrast" => {
            let factor = get_f32(params, "factor")?;
            ContrastFilter::new(factor)
                .map_err(err_to_js)?
                .apply(image)
                .map_err(err_to_js)
        }

        // Filters with params
        "gaussian" => {
            let sigma = get_f32(params, "sigma")?;
            GaussianFilter::new(sigma)
                .map_err(err_to_js)?
                .apply(image)
                .map_err(err_to_js)
        }
        "median" => {
            let size = get_usize(params, "size")?;
            MedianFilter::new(size)
                .map_err(err_to_js)?
                .apply(image)
                .map_err(err_to_js)
        }
        "canny" => {
            let low = get_f32(params, "low")?;
            let high = get_f32(params, "high")?;
            let sigma = get_f32(params, "sigma")?;
            CannyFilter::new(low, high, sigma)
                .map_err(err_to_js)?
                .apply(image)
                .map_err(err_to_js)
        }

        // Transforms
        "crop" => {
            let x = get_u32(params, "x")?;
            let y = get_u32(params, "y")?;
            let w = get_u32(params, "w")?;
            let h = get_u32(params, "h")?;
            CropFilter::new(x, y, w, h)
                .apply(image)
                .map_err(err_to_js)
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
            .map_err(err_to_js)
        }
        "rotate" => {
            let angle = get_f64(params, "angle")?;
            RotateFilter::new(angle, 0, [0, 0, 0])
                .apply(image)
                .map_err(err_to_js)
        }
        "mirror" => {
            let mode_str = get_str(params, "mode")?;
            let mode = match mode_str.as_str() {
                "horizontal" => MirrorMode::Horizontal,
                "vertical" => MirrorMode::Vertical,
                "both" => MirrorMode::Both,
                _ => return Err(JsValue::from_str(&format!("unknown mirror mode: {mode_str}"))),
            };
            MirrorFilter::new(mode)
                .apply(image)
                .map_err(err_to_js)
        }

        _ => Err(JsValue::from_str(&format!("unknown filter: {name}"))),
    }
}

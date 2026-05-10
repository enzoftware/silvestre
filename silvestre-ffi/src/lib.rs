//! C ABI foreign function interface for the silvestre image processing library.
//!
//! All public functions use `extern "C"` and `#[no_mangle]` for stable ABI.
//! Memory is managed via opaque pointers — callers must use [`silvestre_image_free`]
//! to release images. Null pointer arguments return error codes instead of panicking.
//!
//! Error codes: 0 = success, -1 = error (call [`silvestre_last_error`] for details).

use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use std::ptr;
use std::slice;

use silvestre_core::effects::{
    BrightnessFilter, ContrastFilter, GrayscaleFilter, InvertFilter, SepiaFilter,
};
use silvestre_core::filters::{
    BoxBlurFilter, CannyFilter, GaussianFilter, MedianFilter, SharpenFilter, SobelFilter,
};
use silvestre_core::transform::{CropFilter, MirrorFilter, MirrorMode, ResizeFilter, RotateFilter};
use silvestre_core::{ColorSpace, Filter, ImageFormat, SilvestreImage};

/// Success return code.
const OK: i32 = 0;
/// Generic error return code — call `silvestre_last_error` for details.
const ERR: i32 = -1;

// ---------------------------------------------------------------------------
// Thread-local error storage
// ---------------------------------------------------------------------------

thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = const { std::cell::RefCell::new(None) };
}

fn set_last_error(msg: impl std::fmt::Display) {
    let msg = msg.to_string();
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = CString::new(msg).ok();
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

/// Return the version string of the silvestre library.
///
/// # Safety
/// The returned pointer is valid for the lifetime of the program.
#[no_mangle]
pub extern "C" fn silvestre_version() -> *const c_char {
    clear_last_error();
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

// ---------------------------------------------------------------------------
// Image lifecycle
// ---------------------------------------------------------------------------

/// Load an image from a file path.
///
/// Returns a heap-allocated opaque pointer on success, or null on error.
/// The caller must free the returned pointer with [`silvestre_image_free`].
///
/// # Safety
/// `path` must be a valid, null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_load(path: *const c_char) -> *mut SilvestreImage {
    clear_last_error();

    if path.is_null() {
        set_last_error("path is null");
        return ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(path) };
    let path_str = match c_str.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("invalid UTF-8 in path: {e}"));
            return ptr::null_mut();
        }
    };

    match SilvestreImage::load(Path::new(path_str)) {
        Ok(img) => Box::into_raw(Box::new(img)),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Create an image from a raw pixel buffer.
///
/// The pixel data is copied. The buffer must contain exactly `w * h * 4` bytes
/// (RGBA, row-major order).
///
/// Returns a heap-allocated opaque pointer on success, or null on error.
///
/// # Safety
/// `data` must point to at least `len` valid bytes.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_from_buffer(
    data: *const u8,
    len: usize,
    w: u32,
    h: u32,
) -> *mut SilvestreImage {
    clear_last_error();

    if data.is_null() {
        set_last_error("data pointer is null");
        return ptr::null_mut();
    }

    let pixels = unsafe { slice::from_raw_parts(data, len) }.to_vec();

    match SilvestreImage::new(pixels, w, h, ColorSpace::Rgba) {
        Ok(img) => Box::into_raw(Box::new(img)),
        Err(e) => {
            set_last_error(e);
            ptr::null_mut()
        }
    }
}

/// Free an image previously returned by `silvestre_image_load` or
/// `silvestre_image_from_buffer`.
///
/// Passing null is a no-op. Passing the same pointer twice is undefined behaviour.
///
/// # Safety
/// `img` must be a pointer previously returned by this library, or null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_free(img: *mut SilvestreImage) {
    clear_last_error();
    if !img.is_null() {
        drop(unsafe { Box::from_raw(img) });
    }
}

// ---------------------------------------------------------------------------
// Image queries
// ---------------------------------------------------------------------------

/// Return the width of the image in pixels, or 0 if `img` is null.
///
/// # Safety
/// `img` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_width(img: *const SilvestreImage) -> u32 {
    clear_last_error();
    if img.is_null() {
        set_last_error("image pointer is null");
        return 0;
    }
    unsafe { &*img }.width()
}

/// Return the height of the image in pixels, or 0 if `img` is null.
///
/// # Safety
/// `img` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_height(img: *const SilvestreImage) -> u32 {
    clear_last_error();
    if img.is_null() {
        set_last_error("image pointer is null");
        return 0;
    }
    unsafe { &*img }.height()
}

/// Return a pointer to the raw pixel data of the image.
///
/// The returned pointer is valid until the image is freed or mutated
/// (e.g. by [`silvestre_apply_filter`]). Callers should copy the data
/// before calling any mutating operation.
/// Returns null if `img` is null.
///
/// # Safety
/// `img` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_pixels(img: *const SilvestreImage) -> *const u8 {
    clear_last_error();
    if img.is_null() {
        set_last_error("image pointer is null");
        return ptr::null();
    }
    unsafe { &*img }.pixels().as_ptr()
}

/// Return the size of the pixel buffer in bytes, or 0 if `img` is null.
///
/// # Safety
/// `img` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_pixels_len(img: *const SilvestreImage) -> usize {
    clear_last_error();
    if img.is_null() {
        set_last_error("image pointer is null");
        return 0;
    }
    unsafe { &*img }.pixels().len()
}

// ---------------------------------------------------------------------------
// Save
// ---------------------------------------------------------------------------

/// Save the image to a file.
///
/// `format` is a null-terminated string: `"png"`, `"jpeg"`, or `"bmp"`.
/// If `format` is null, the format is inferred from the file extension.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// `img`, `path` must be valid pointers. `format` may be null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_image_save(
    img: *const SilvestreImage,
    path: *const c_char,
    format: *const c_char,
) -> i32 {
    clear_last_error();

    if img.is_null() {
        set_last_error("image pointer is null");
        return ERR;
    }
    if path.is_null() {
        set_last_error("path is null");
        return ERR;
    }

    let image = unsafe { &*img };
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("invalid UTF-8 in path: {e}"));
            return ERR;
        }
    };

    let result = if format.is_null() {
        image.save(Path::new(path_str))
    } else {
        let fmt_str = match unsafe { CStr::from_ptr(format) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                set_last_error(format!("invalid UTF-8 in format: {e}"));
                return ERR;
            }
        };
        match parse_image_format(fmt_str) {
            Some(fmt) => image.save_with_format(Path::new(path_str), fmt),
            None => {
                set_last_error(format!("unknown image format: {fmt_str}"));
                return ERR;
            }
        }
    };

    match result {
        Ok(()) => OK,
        Err(e) => {
            set_last_error(e);
            ERR
        }
    }
}

// ---------------------------------------------------------------------------
// Filter application
// ---------------------------------------------------------------------------

/// Apply a named filter to the image, replacing its contents in place.
///
/// `filter_name` is a null-terminated string identifying the filter.
/// `params` is a null-terminated JSON string with filter parameters (may be null
/// for filters that take no parameters).
///
/// Supported filters and their JSON params:
///
/// | filter_name   | params (JSON)                                           |
/// |---------------|---------------------------------------------------------|
/// | `grayscale`   | *(none)*                                                |
/// | `invert`      | *(none)*                                                |
/// | `sepia`       | *(none)*                                                |
/// | `brightness`  | `{"delta": <i32>}`                                      |
/// | `contrast`    | `{"factor": <f32>}`                                     |
/// | `gaussian`    | `{"sigma": <f32>}`                                      |
/// | `median`      | `{"size": <usize>}`                                     |
/// | `sharpen`     | *(none)*                                                |
/// | `box_blur`    | *(none)*                                                |
/// | `sobel`       | *(none)*                                                |
/// | `canny`       | `{"low": <f32>, "high": <f32>, "sigma": <f32>}`        |
/// | `crop`        | `{"x": <u32>, "y": <u32>, "w": <u32>, "h": <u32>}`     |
/// | `resize`      | `{"w": <u32>, "h": <u32>}`                              |
/// | `rotate`      | `{"angle": <f64>}`                                      |
/// | `mirror`      | `{"mode": "horizontal"|"vertical"|"both"}`              |
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
/// `img` must be a valid mutable pointer. `filter_name` must be valid.
/// `params` may be null.
#[no_mangle]
pub unsafe extern "C" fn silvestre_apply_filter(
    img: *mut SilvestreImage,
    filter_name: *const c_char,
    params: *const c_char,
) -> i32 {
    clear_last_error();

    if img.is_null() {
        set_last_error("image pointer is null");
        return ERR;
    }
    if filter_name.is_null() {
        set_last_error("filter_name is null");
        return ERR;
    }

    let name = match unsafe { CStr::from_ptr(filter_name) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_last_error(format!("invalid UTF-8 in filter_name: {e}"));
            return ERR;
        }
    };

    let params_str = if params.is_null() {
        ""
    } else {
        match unsafe { CStr::from_ptr(params) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                set_last_error(format!("invalid UTF-8 in params: {e}"));
                return ERR;
            }
        }
    };

    let image = unsafe { &*img };
    let result = apply_named_filter(name, image, params_str);

    match result {
        Ok(new_image) => {
            unsafe { *img = new_image };
            OK
        }
        Err(msg) => {
            set_last_error(msg);
            ERR
        }
    }
}

// ---------------------------------------------------------------------------
// Error retrieval
// ---------------------------------------------------------------------------

/// Return a pointer to the last error message for the current thread.
///
/// Returns null if no error has occurred. The pointer is valid until the next
/// FFI call on the same thread. Callers should copy the string immediately
/// if they need to preserve it.
///
/// # Safety
/// Must be called from the same thread that triggered the error.
#[no_mangle]
pub extern "C" fn silvestre_last_error() -> *const c_char {
    LAST_ERROR.with(|cell| {
        let borrow = cell.borrow();
        match borrow.as_ref() {
            Some(cstr) => cstr.as_ptr(),
            None => ptr::null(),
        }
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn parse_image_format(s: &str) -> Option<ImageFormat> {
    match s.to_ascii_lowercase().as_str() {
        "png" => Some(ImageFormat::Png),
        "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
        "bmp" => Some(ImageFormat::Bmp),
        _ => None,
    }
}

fn apply_named_filter(
    name: &str,
    image: &SilvestreImage,
    params: &str,
) -> Result<SilvestreImage, String> {
    match name {
        // Effects (no params)
        "grayscale" => GrayscaleFilter.apply(image).map_err(|e| e.to_string()),
        "invert" => InvertFilter.apply(image).map_err(|e| e.to_string()),
        "sepia" => SepiaFilter.apply(image).map_err(|e| e.to_string()),
        "sharpen" => SharpenFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(|e| e.to_string()),
        "box_blur" => BoxBlurFilter::new()
            .and_then(|f| f.apply(image))
            .map_err(|e| e.to_string()),
        "sobel" => SobelFilter::new().apply(image).map_err(|e| e.to_string()),

        // Effects with params
        "brightness" => {
            let delta = parse_param_i32(params, "delta")?;
            BrightnessFilter::new(delta)
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "contrast" => {
            let factor = parse_param_f32(params, "factor")?;
            ContrastFilter::new(factor)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }

        // Filters with params
        "gaussian" => {
            let sigma = parse_param_f32(params, "sigma")?;
            GaussianFilter::new(sigma)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "median" => {
            let size = parse_param_usize(params, "size")?;
            MedianFilter::new(size)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "canny" => {
            let low = parse_param_f32(params, "low")?;
            let high = parse_param_f32(params, "high")?;
            let sigma = parse_param_f32(params, "sigma")?;
            CannyFilter::new(low, high, sigma)
                .map_err(|e| e.to_string())?
                .apply(image)
                .map_err(|e| e.to_string())
        }

        // Transforms
        "crop" => {
            let x = parse_param_u32(params, "x")?;
            let y = parse_param_u32(params, "y")?;
            let w = parse_param_u32(params, "w")?;
            let h = parse_param_u32(params, "h")?;
            CropFilter::new(x, y, w, h)
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "resize" => {
            let w = parse_param_u32(params, "w")?;
            let h = parse_param_u32(params, "h")?;
            ResizeFilter::new(
                w,
                h,
                silvestre_core::transform::Interpolation::Bilinear,
            )
            .apply(image)
            .map_err(|e| e.to_string())
        }
        "rotate" => {
            let angle = parse_param_f64(params, "angle")?;
            RotateFilter::new(angle, 0, [0, 0, 0])
                .apply(image)
                .map_err(|e| e.to_string())
        }
        "mirror" => {
            let mode_str = parse_param_str(params, "mode")?;
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
// Minimal JSON parameter parsing (avoids adding serde_json dependency)
// ---------------------------------------------------------------------------

/// Extract a string value for `key` from a simple flat JSON object.
///
/// This is a minimal parser for flat `{"key": value}` objects. It does not
/// handle escaped quotes in string values or deeply nested structures.
fn extract_json_value(json: &str, key: &str) -> Result<String, String> {
    if json.is_empty() {
        return Err(format!("missing required parameter: {key}"));
    }

    // Search for `"key"` followed by optional whitespace and a colon.
    let search = format!("\"{key}\"");
    let mut search_from = 0;
    let after_colon = loop {
        let pos = json[search_from..]
            .find(&search)
            .map(|p| p + search_from)
            .ok_or_else(|| format!("missing required parameter: {key}"))?;

        let rest = json[pos + search.len()..].trim_start();
        if let Some(after) = rest.strip_prefix(':') {
            break after.trim_start();
        }
        // This occurrence wasn't a key (no colon after it), keep searching.
        search_from = pos + search.len();
    };

    // If the value is a quoted string
    if after_colon.starts_with('"') {
        let content = &after_colon[1..];
        let end = content
            .find('"')
            .ok_or_else(|| format!("unterminated string for parameter: {key}"))?;
        return Ok(content[..end].to_string());
    }

    // Otherwise it's a number/boolean — take until comma, brace, or end.
    let end = after_colon
        .find(|c: char| c == ',' || c == '}' || c == ']')
        .unwrap_or(after_colon.len());

    let value = after_colon[..end].trim();
    if value.is_empty() {
        return Err(format!("empty value for parameter: {key}"));
    }
    Ok(value.to_string())
}

fn parse_param_i32(json: &str, key: &str) -> Result<i32, String> {
    extract_json_value(json, key)?
        .parse::<i32>()
        .map_err(|e| format!("invalid i32 for {key}: {e}"))
}

fn parse_param_u32(json: &str, key: &str) -> Result<u32, String> {
    extract_json_value(json, key)?
        .parse::<u32>()
        .map_err(|e| format!("invalid u32 for {key}: {e}"))
}

fn parse_param_usize(json: &str, key: &str) -> Result<usize, String> {
    extract_json_value(json, key)?
        .parse::<usize>()
        .map_err(|e| format!("invalid usize for {key}: {e}"))
}

fn parse_param_f32(json: &str, key: &str) -> Result<f32, String> {
    extract_json_value(json, key)?
        .parse::<f32>()
        .map_err(|e| format!("invalid f32 for {key}: {e}"))
}

fn parse_param_f64(json: &str, key: &str) -> Result<f64, String> {
    extract_json_value(json, key)?
        .parse::<f64>()
        .map_err(|e| format!("invalid f64 for {key}: {e}"))
}

fn parse_param_str(json: &str, key: &str) -> Result<String, String> {
    extract_json_value(json, key)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Helper: create a 2x2 RGBA test image via FFI
    unsafe fn make_test_image() -> *mut SilvestreImage {
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 255, 0, 255, // yellow
        ];
        let ptr =
            unsafe { silvestre_image_from_buffer(pixels.as_ptr(), pixels.len(), 2, 2) };
        assert!(!ptr.is_null(), "failed to create test image");
        ptr
    }

    // -- Version --

    #[test]
    fn test_version_is_not_null() {
        let v = silvestre_version();
        assert!(!v.is_null());
        let cstr = unsafe { CStr::from_ptr(v) };
        assert!(!cstr.to_str().unwrap().is_empty());
    }

    // -- Image lifecycle --

    #[test]
    fn test_image_from_buffer_and_free() {
        unsafe {
            let img = make_test_image();
            assert_eq!(silvestre_image_width(img), 2);
            assert_eq!(silvestre_image_height(img), 2);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_image_from_buffer_null_data() {
        let img = unsafe { silvestre_image_from_buffer(ptr::null(), 0, 1, 1) };
        assert!(img.is_null());
        let err = silvestre_last_error();
        assert!(!err.is_null());
        let msg = unsafe { CStr::from_ptr(err) }.to_str().unwrap();
        assert!(msg.contains("null"), "error: {msg}");
    }

    #[test]
    fn test_image_from_buffer_size_mismatch() {
        let pixels: Vec<u8> = vec![0; 4]; // 4 bytes but asking for 2x2 (needs 16)
        let img =
            unsafe { silvestre_image_from_buffer(pixels.as_ptr(), pixels.len(), 2, 2) };
        assert!(img.is_null());
        let err = silvestre_last_error();
        assert!(!err.is_null());
    }

    #[test]
    fn test_free_null_is_noop() {
        unsafe { silvestre_image_free(ptr::null_mut()) };
    }

    // -- Null pointer guards --

    #[test]
    fn test_width_null() {
        assert_eq!(unsafe { silvestre_image_width(ptr::null()) }, 0);
    }

    #[test]
    fn test_height_null() {
        assert_eq!(unsafe { silvestre_image_height(ptr::null()) }, 0);
    }

    #[test]
    fn test_pixels_null() {
        assert!(unsafe { silvestre_image_pixels(ptr::null()) }.is_null());
    }

    #[test]
    fn test_pixels_len_null() {
        assert_eq!(unsafe { silvestre_image_pixels_len(ptr::null()) }, 0);
    }

    // -- Pixel data access --

    #[test]
    fn test_image_pixels() {
        unsafe {
            let img = make_test_image();
            let p = silvestre_image_pixels(img);
            assert!(!p.is_null());
            let len = silvestre_image_pixels_len(img);
            assert_eq!(len, 16); // 2*2*4
            // First pixel is red
            assert_eq!(*p, 255);
            assert_eq!(*p.add(1), 0);
            assert_eq!(*p.add(2), 0);
            assert_eq!(*p.add(3), 255);
            silvestre_image_free(img);
        }
    }

    // -- Save with null args --

    #[test]
    fn test_save_null_image() {
        let path = CString::new("/tmp/test.png").unwrap();
        let rc = unsafe {
            silvestre_image_save(ptr::null(), path.as_ptr(), ptr::null())
        };
        assert_eq!(rc, ERR);
    }

    #[test]
    fn test_save_null_path() {
        unsafe {
            let img = make_test_image();
            let rc = silvestre_image_save(img, ptr::null(), ptr::null());
            assert_eq!(rc, ERR);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        unsafe {
            let img = make_test_image();
            let tmp = std::env::temp_dir().join("silvestre_ffi_test.png");
            let path = CString::new(tmp.to_str().unwrap()).unwrap();
            let fmt = CString::new("png").unwrap();
            let rc = silvestre_image_save(img, path.as_ptr(), fmt.as_ptr());
            assert_eq!(rc, OK);

            let loaded = silvestre_image_load(path.as_ptr());
            assert!(!loaded.is_null());
            assert_eq!(silvestre_image_width(loaded), 2);
            assert_eq!(silvestre_image_height(loaded), 2);

            silvestre_image_free(loaded);
            silvestre_image_free(img);
            let _ = std::fs::remove_file(&tmp);
        }
    }

    // -- Load errors --

    #[test]
    fn test_load_null_path() {
        let img = unsafe { silvestre_image_load(ptr::null()) };
        assert!(img.is_null());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let path = CString::new("/tmp/does_not_exist_silvestre.png").unwrap();
        let img = unsafe { silvestre_image_load(path.as_ptr()) };
        assert!(img.is_null());
        let err = silvestre_last_error();
        assert!(!err.is_null());
    }

    // -- Apply filter: null guards --

    #[test]
    fn test_apply_filter_null_image() {
        let name = CString::new("grayscale").unwrap();
        let rc =
            unsafe { silvestre_apply_filter(ptr::null_mut(), name.as_ptr(), ptr::null()) };
        assert_eq!(rc, ERR);
    }

    #[test]
    fn test_apply_filter_null_name() {
        unsafe {
            let img = make_test_image();
            let rc = silvestre_apply_filter(img, ptr::null(), ptr::null());
            assert_eq!(rc, ERR);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_filter_unknown() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("nonexistent_filter").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, ERR);
            let err_ptr = silvestre_last_error();
            let msg = CStr::from_ptr(err_ptr).to_str().unwrap();
            assert!(msg.contains("unknown filter"), "error: {msg}");
            silvestre_image_free(img);
        }
    }

    // -- Apply filters: happy paths --

    #[test]
    fn test_apply_grayscale() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("grayscale").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_invert() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("invert").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, OK);

            // First pixel was red (255,0,0,255), inverted should be (0,255,255,255)
            let p = silvestre_image_pixels(img);
            assert_eq!(*p, 0);
            assert_eq!(*p.add(1), 255);
            assert_eq!(*p.add(2), 255);
            assert_eq!(*p.add(3), 255); // alpha preserved

            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_sepia() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("sepia").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_brightness() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("brightness").unwrap();
            let params = CString::new(r#"{"delta": 10}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_contrast() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("contrast").unwrap();
            let params = CString::new(r#"{"factor": 1.5}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_gaussian() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("gaussian").unwrap();
            let params = CString::new(r#"{"sigma": 1.0}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_median() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("median").unwrap();
            let params = CString::new(r#"{"size": 3}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_sharpen() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("sharpen").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_box_blur() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("box_blur").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_crop() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("crop").unwrap();
            let params = CString::new(r#"{"x": 0, "y": 0, "w": 1, "h": 1}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            assert_eq!(silvestre_image_width(img), 1);
            assert_eq!(silvestre_image_height(img), 1);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_resize() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("resize").unwrap();
            let params = CString::new(r#"{"w": 4, "h": 4}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            assert_eq!(silvestre_image_width(img), 4);
            assert_eq!(silvestre_image_height(img), 4);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_rotate() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("rotate").unwrap();
            let params = CString::new(r#"{"angle": 90.0}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_mirror() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("mirror").unwrap();
            let params = CString::new(r#"{"mode": "horizontal"}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
        }
    }

    // -- Filter param errors --

    #[test]
    fn test_apply_brightness_missing_param() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("brightness").unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), ptr::null());
            assert_eq!(rc, ERR);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_gaussian_invalid_sigma() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("gaussian").unwrap();
            let params = CString::new(r#"{"sigma": "abc"}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, ERR);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_apply_mirror_invalid_mode() {
        unsafe {
            let img = make_test_image();
            let name = CString::new("mirror").unwrap();
            let params = CString::new(r#"{"mode": "diagonal"}"#).unwrap();
            let rc = silvestre_apply_filter(img, name.as_ptr(), params.as_ptr());
            assert_eq!(rc, ERR);
            let err_ptr = silvestre_last_error();
            let msg = CStr::from_ptr(err_ptr).to_str().unwrap();
            assert!(msg.contains("unknown mirror mode"), "error: {msg}");
            silvestre_image_free(img);
        }
    }

    // -- Error message lifecycle --

    #[test]
    fn test_last_error_null_when_no_error() {
        clear_last_error();
        let err = silvestre_last_error();
        assert!(err.is_null());
    }

    #[test]
    fn test_last_error_set_and_cleared() {
        // Trigger an error
        let _ = unsafe { silvestre_image_load(ptr::null()) };
        let err = silvestre_last_error();
        assert!(!err.is_null());

        // Successful operation clears it
        unsafe {
            let img = make_test_image();
            // After successful from_buffer, last error should be cleared
            assert!(silvestre_last_error().is_null());
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_query_functions_clear_stale_error() {
        unsafe {
            // Trigger an error
            let _ = silvestre_image_load(ptr::null());
            assert!(!silvestre_last_error().is_null());

            // A successful query call should clear the stale error
            let img = make_test_image();
            silvestre_image_width(img);
            assert!(silvestre_last_error().is_null());

            // Trigger error again, then check height clears it
            let _ = silvestre_image_load(ptr::null());
            assert!(!silvestre_last_error().is_null());
            silvestre_image_height(img);
            assert!(silvestre_last_error().is_null());

            // Same for pixels and pixels_len
            let _ = silvestre_image_load(ptr::null());
            silvestre_image_pixels(img);
            assert!(silvestre_last_error().is_null());

            let _ = silvestre_image_load(ptr::null());
            silvestre_image_pixels_len(img);
            assert!(silvestre_last_error().is_null());

            silvestre_image_free(img);
        }
    }

    // -- Save format variants --

    #[test]
    fn test_save_unknown_format() {
        unsafe {
            let img = make_test_image();
            let path = CString::new("/tmp/silvestre_ffi_test.xyz").unwrap();
            let fmt = CString::new("xyz").unwrap();
            let rc = silvestre_image_save(img, path.as_ptr(), fmt.as_ptr());
            assert_eq!(rc, ERR);
            silvestre_image_free(img);
        }
    }

    #[test]
    fn test_save_inferred_format() {
        unsafe {
            let img = make_test_image();
            let tmp = std::env::temp_dir().join("silvestre_ffi_infer_test.bmp");
            let path = CString::new(tmp.to_str().unwrap()).unwrap();
            let rc = silvestre_image_save(img, path.as_ptr(), ptr::null());
            assert_eq!(rc, OK);
            silvestre_image_free(img);
            let _ = std::fs::remove_file(&tmp);
        }
    }

    // -- JSON parsing edge cases --

    #[test]
    fn test_extract_json_value_basic() {
        let val = extract_json_value(r#"{"delta": 42}"#, "delta").unwrap();
        assert_eq!(val, "42");
    }

    #[test]
    fn test_extract_json_value_string() {
        let val = extract_json_value(r#"{"mode": "horizontal"}"#, "mode").unwrap();
        assert_eq!(val, "horizontal");
    }

    #[test]
    fn test_extract_json_value_missing_key() {
        let result = extract_json_value(r#"{"delta": 42}"#, "sigma");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_value_empty() {
        let result = extract_json_value("", "key");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_json_multiple_keys() {
        let json = r#"{"x": 10, "y": 20, "w": 100, "h": 200}"#;
        assert_eq!(extract_json_value(json, "x").unwrap(), "10");
        assert_eq!(extract_json_value(json, "y").unwrap(), "20");
        assert_eq!(extract_json_value(json, "w").unwrap(), "100");
        assert_eq!(extract_json_value(json, "h").unwrap(), "200");
    }

    #[test]
    fn test_extract_json_negative_value() {
        let val = extract_json_value(r#"{"delta": -50}"#, "delta").unwrap();
        assert_eq!(val, "-50");
    }

    #[test]
    fn test_extract_json_float_value() {
        let val = extract_json_value(r#"{"sigma": 1.5}"#, "sigma").unwrap();
        assert_eq!(val, "1.5");
    }
}

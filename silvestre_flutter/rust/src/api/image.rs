use silvestre_core::{ColorSpace, ImageFormat, SilvestreImage};

/// Opaque wrapper around `SilvestreImage`.
///
/// FRB treats this as an opaque type, meaning Dart code holds a reference
/// to it but cannot inspect its fields directly. Public functions below
/// provide the bridge between Dart and the inner image.
pub struct SilvestreImageWrapper {
    pub(crate) inner: SilvestreImage,
}

// ---- Construction --------------------------------------------------------

/// Load an image from a file path (PNG, JPEG, BMP).
pub fn load_image_from_path(path: String) -> Result<SilvestreImageWrapper, String> {
    let inner = SilvestreImage::load(&path).map_err(|e| e.to_string())?;
    Ok(SilvestreImageWrapper { inner })
}

/// Load an image from raw encoded bytes (PNG, JPEG, BMP).
pub fn load_image_from_bytes(bytes: Vec<u8>) -> Result<SilvestreImageWrapper, String> {
    let inner = SilvestreImage::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    Ok(SilvestreImageWrapper { inner })
}

/// Create an image from raw pixel data.
///
/// `color_space` must be one of: `"rgba"`, `"rgb"`, `"grayscale"`.
pub fn create_image(
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    color_space: String,
) -> Result<SilvestreImageWrapper, String> {
    let cs = parse_color_space(&color_space)?;
    let inner = SilvestreImage::new(pixels, width, height, cs).map_err(|e| e.to_string())?;
    Ok(SilvestreImageWrapper { inner })
}

// ---- Properties ----------------------------------------------------------

/// Get the image width in pixels.
pub fn image_width(img: &SilvestreImageWrapper) -> u32 {
    img.inner.width()
}

/// Get the image height in pixels.
pub fn image_height(img: &SilvestreImageWrapper) -> u32 {
    img.inner.height()
}

/// Get the image color space as a string (`"rgba"`, `"rgb"`, or `"grayscale"`).
pub fn image_color_space(img: &SilvestreImageWrapper) -> String {
    match img.inner.color_space() {
        ColorSpace::Rgba => "rgba".to_string(),
        ColorSpace::Rgb => "rgb".to_string(),
        ColorSpace::Grayscale => "grayscale".to_string(),
    }
}

/// Get a copy of the raw pixel data.
pub fn image_pixels(img: &SilvestreImageWrapper) -> Vec<u8> {
    img.inner.pixels().to_vec()
}

// ---- I/O -----------------------------------------------------------------

/// Save the image to a file path in the specified format.
///
/// `format` must be one of: `"png"`, `"jpeg"` / `"jpg"`, `"bmp"`.
pub fn save_image(
    img: &SilvestreImageWrapper,
    path: String,
    format: String,
) -> Result<(), String> {
    let fmt = parse_format(&format)?;
    img.inner
        .save_with_format(&path, fmt)
        .map_err(|e| e.to_string())
}

/// Encode the image to in-memory bytes in the specified format.
///
/// `format` must be one of: `"png"`, `"jpeg"` / `"jpg"`, `"bmp"`.
pub fn encode_image(img: &SilvestreImageWrapper, format: String) -> Result<Vec<u8>, String> {
    let fmt = parse_format(&format)?;
    img.inner.encode_to_memory(fmt).map_err(|e| e.to_string())
}

// ---- Helpers -------------------------------------------------------------

fn parse_color_space(s: &str) -> Result<ColorSpace, String> {
    match s.to_ascii_lowercase().as_str() {
        "rgba" => Ok(ColorSpace::Rgba),
        "rgb" => Ok(ColorSpace::Rgb),
        "grayscale" => Ok(ColorSpace::Grayscale),
        _ => Err(format!("unknown color space: {s}")),
    }
}

fn parse_format(s: &str) -> Result<ImageFormat, String> {
    match s.to_ascii_lowercase().as_str() {
        "png" => Ok(ImageFormat::Png),
        "jpg" | "jpeg" => Ok(ImageFormat::Jpeg),
        "bmp" => Ok(ImageFormat::Bmp),
        _ => Err(format!("unsupported format: {s}")),
    }
}

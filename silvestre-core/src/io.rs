use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageFormat as CrateImageFormat};

use crate::{ColorSpace, Result, SilvestreError, SilvestreImage};

/// Supported image file formats for loading and saving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Bmp,
}

impl ImageFormat {
    /// Convert to the `image` crate's format type.
    fn to_crate_format(self) -> CrateImageFormat {
        match self {
            Self::Png => CrateImageFormat::Png,
            Self::Jpeg => CrateImageFormat::Jpeg,
            Self::Bmp => CrateImageFormat::Bmp,
        }
    }

    /// Infer the format from a file extension.
    fn from_extension(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        match ext.as_deref() {
            Some("png") => Ok(Self::Png),
            Some("jpg" | "jpeg") => Ok(Self::Jpeg),
            Some("bmp") => Ok(Self::Bmp),
            _ => Err(SilvestreError::InvalidParameter(format!(
                "unsupported or missing file extension: {}",
                path.display()
            ))),
        }
    }
}

/// Convert a `DynamicImage` into a `SilvestreImage`, preserving the
/// original color space when possible.
fn dynamic_to_silvestre(img: DynamicImage) -> Result<SilvestreImage> {
    let (width, height) = img.dimensions();
    match img {
        DynamicImage::ImageLuma8(gray) => {
            SilvestreImage::new(gray.into_raw(), width, height, ColorSpace::Grayscale)
        }
        DynamicImage::ImageRgb8(rgb) => {
            SilvestreImage::new(rgb.into_raw(), width, height, ColorSpace::Rgb)
        }
        other => {
            // Everything else (RGBA, 16-bit, palette, etc.) → RGBA8
            let rgba = other.into_rgba8();
            SilvestreImage::new(rgba.into_raw(), width, height, ColorSpace::Rgba)
        }
    }
}

impl SilvestreImage {
    /// Map the image's color space to the corresponding `image::ColorType`.
    fn to_image_color_type(&self) -> image::ColorType {
        match self.color_space() {
            ColorSpace::Rgba => image::ColorType::Rgba8,
            ColorSpace::Rgb => image::ColorType::Rgb8,
            ColorSpace::Grayscale => image::ColorType::L8,
        }
    }

    /// Load an image from a file path.
    ///
    /// The color space is preserved when the source is Grayscale or RGB;
    /// all other formats are converted to RGBA.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let img = image::open(path)?;
        dynamic_to_silvestre(img)
    }

    /// Save the image to a file path.
    ///
    /// The format is inferred from the file extension. Use
    /// [`save_with_format`](Self::save_with_format) to specify it explicitly.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let format = ImageFormat::from_extension(path.as_ref())?;
        self.save_with_format(path, format)
    }

    /// Save the image to a file path with an explicit format.
    ///
    /// The format is used directly rather than being inferred from the
    /// file extension, so the extension and format need not match.
    /// Writes are streamed through a `BufWriter` to avoid buffering the
    /// entire encoded image in memory.
    pub fn save_with_format(&self, path: impl AsRef<Path>, format: ImageFormat) -> Result<()> {
        let color_type = self.to_image_color_type();
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        image::write_buffer_with_format(
            &mut writer,
            self.pixels(),
            self.width(),
            self.height(),
            color_type,
            format.to_crate_format(),
        )?;
        Ok(())
    }

    /// Load an image from an in-memory byte buffer.
    ///
    /// The format is auto-detected from the buffer contents.
    pub fn load_from_memory(bytes: &[u8]) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        dynamic_to_silvestre(img)
    }

    /// Encode the image into an in-memory byte buffer in the given format.
    pub fn encode_to_memory(&self, format: ImageFormat) -> Result<Vec<u8>> {
        let color_type = self.to_image_color_type();
        let mut buf = Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut buf,
            self.pixels(),
            self.width(),
            self.height(),
            color_type,
            format.to_crate_format(),
        )?;
        Ok(buf.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a small 2×2 RGBA test image with known pixel values.
    fn test_image_rgba() -> SilvestreImage {
        #[rustfmt::skip]
        let pixels = vec![
            255,   0,   0, 255, // red
              0, 255,   0, 255, // green
              0,   0, 255, 255, // blue
            255, 255, 255, 255, // white
        ];
        SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgba).unwrap()
    }

    /// Helper: create a small 2×2 RGB test image.
    fn test_image_rgb() -> SilvestreImage {
        #[rustfmt::skip]
        let pixels = vec![
            255,   0,   0, // red
              0, 255,   0, // green
              0,   0, 255, // blue
            255, 255, 255, // white
        ];
        SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgb).unwrap()
    }

    /// Helper: create a small 2×2 Grayscale test image.
    fn test_image_grayscale() -> SilvestreImage {
        let pixels = vec![0, 85, 170, 255];
        SilvestreImage::new(pixels, 2, 2, ColorSpace::Grayscale).unwrap()
    }

    // ── Round-trip tests (lossless formats) ─────────────────────────

    #[test]
    fn round_trip_png_rgba() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("round_trip.png");
        let img = test_image_rgba();
        img.save(&path).unwrap();

        let loaded = SilvestreImage::load(&path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        assert_eq!(loaded.color_space(), ColorSpace::Rgba);
        assert_eq!(loaded.pixels(), img.pixels());
    }

    #[test]
    fn round_trip_png_rgb() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("round_trip_rgb.png");
        let img = test_image_rgb();
        img.save(&path).unwrap();

        let loaded = SilvestreImage::load(&path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        assert_eq!(loaded.color_space(), ColorSpace::Rgb);
        assert_eq!(loaded.pixels(), img.pixels());
    }

    #[test]
    fn round_trip_png_grayscale() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("round_trip_gray.png");
        let img = test_image_grayscale();
        img.save(&path).unwrap();

        let loaded = SilvestreImage::load(&path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        assert_eq!(loaded.color_space(), ColorSpace::Grayscale);
        assert_eq!(loaded.pixels(), img.pixels());
    }

    #[test]
    fn round_trip_bmp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("round_trip.bmp");
        let img = test_image_rgba();
        img.save(&path).unwrap();

        let loaded = SilvestreImage::load(&path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        assert_eq!(loaded.pixels(), img.pixels());
    }

    // ── JPEG (lossy — only check dimensions) ────────────────────────

    #[test]
    fn save_and_load_jpeg() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jpg");
        let img = test_image_rgb();
        img.save(&path).unwrap();

        let loaded = SilvestreImage::load(&path).unwrap();
        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
        // JPEG is lossy so we don't compare pixel data.
    }

    // ── In-memory round-trips ───────────────────────────────────────

    #[test]
    fn encode_decode_png_memory() {
        let img = test_image_rgba();
        let bytes = img.encode_to_memory(ImageFormat::Png).unwrap();
        let loaded = SilvestreImage::load_from_memory(&bytes).unwrap();

        assert_eq!(loaded.width(), img.width());
        assert_eq!(loaded.height(), img.height());
        assert_eq!(loaded.pixels(), img.pixels());
    }

    #[test]
    fn encode_decode_bmp_memory() {
        let img = test_image_rgb();
        let bytes = img.encode_to_memory(ImageFormat::Bmp).unwrap();
        let loaded = SilvestreImage::load_from_memory(&bytes).unwrap();

        assert_eq!(loaded.width(), img.width());
        assert_eq!(loaded.height(), img.height());
        assert_eq!(loaded.pixels(), img.pixels());
    }

    #[test]
    fn encode_jpeg_memory() {
        let img = test_image_rgb();
        let bytes = img.encode_to_memory(ImageFormat::Jpeg).unwrap();
        let loaded = SilvestreImage::load_from_memory(&bytes).unwrap();

        assert_eq!(loaded.width(), 2);
        assert_eq!(loaded.height(), 2);
    }

    // ── Error cases ─────────────────────────────────────────────────

    #[test]
    fn load_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let result = SilvestreImage::load(dir.path().join("does_not_exist.png"));
        assert!(result.is_err());
    }

    #[test]
    fn load_corrupt_data() {
        let result = SilvestreImage::load_from_memory(&[0, 1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn save_unsupported_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.xyz");
        let img = test_image_rgba();
        let result = img.save(&path);
        assert!(result.is_err());
    }

    #[test]
    fn image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension(Path::new("a.png")).unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_extension(Path::new("a.jpg")).unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension(Path::new("a.jpeg")).unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension(Path::new("a.bmp")).unwrap(), ImageFormat::Bmp);
        assert!(ImageFormat::from_extension(Path::new("a.tiff")).is_err());
        assert!(ImageFormat::from_extension(Path::new("no_ext")).is_err());
    }
}

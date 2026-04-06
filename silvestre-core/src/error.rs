use thiserror::Error;

/// Errors that can occur during image processing.
#[derive(Debug, Error)]
pub enum SilvestreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("image decoding error: {0}")]
    ImageDecode(#[from] image::ImageError),

    #[error("invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },

    #[error("raw pixel buffer size mismatch: expected {expected} bytes, got {got}")]
    BufferSizeMismatch { expected: usize, got: usize },

    #[error("pixel coordinates out of bounds: ({x}, {y}) in {width}x{height} image")]
    OutOfBounds { x: u32, y: u32, width: u32, height: u32 },

    #[error("unsupported color space: {0:?}")]
    UnsupportedColorSpace(crate::ColorSpace),

    #[error("channel count mismatch: expected {expected}, got {got}")]
    ChannelMismatch { expected: usize, got: usize },

    #[error("{0}")]
    InvalidParameter(String),
}

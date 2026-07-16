use thiserror::Error;

/// Errors that can occur during image processing.
///
/// Every fallible operation in silvestre returns [`crate::Result`], which wraps
/// this type. Variants carry enough context (dimensions, offsets, expected vs.
/// actual sizes) to diagnose the failure without additional logging.
#[derive(Debug, Error)]
pub enum SilvestreError {
    /// An underlying I/O operation failed (e.g. reading or writing a file).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The `image` crate could not decode or encode the image data.
    #[error("image decoding error: {0}")]
    ImageDecode(#[from] image::ImageError),

    /// The requested `width`×`height` is invalid, typically because the pixel
    /// count overflows `usize`.
    #[error("invalid dimensions: {width}x{height}")]
    InvalidDimensions {
        /// Requested image width in pixels.
        width: u32,
        /// Requested image height in pixels.
        height: u32,
    },

    /// A raw pixel buffer did not match the size implied by the image's
    /// dimensions and color space.
    #[error("raw pixel buffer size mismatch: expected {expected} bytes, got {got}")]
    BufferSizeMismatch {
        /// Number of bytes the buffer was expected to contain.
        expected: usize,
        /// Number of bytes the buffer actually contained.
        got: usize,
    },

    /// Pixel coordinates fell outside the image bounds.
    #[error("pixel coordinates out of bounds: ({x}, {y}) in {width}x{height} image.")]
    OutOfBounds {
        /// The out-of-range x coordinate.
        x: u32,
        /// The out-of-range y coordinate.
        y: u32,
        /// Image width the coordinate was checked against.
        width: u32,
        /// Image height the coordinate was checked against.
        height: u32,
    },

    /// An operation does not support the image's color space.
    #[error("unsupported color space: {0:?}")]
    UnsupportedColorSpace(crate::ColorSpace),

    /// A pixel value had a different number of channels than the image's color
    /// space requires.
    #[error("channel count mismatch: expected {expected}, got {got}")]
    ChannelMismatch {
        /// Number of channels the color space requires.
        expected: usize,
        /// Number of channels supplied.
        got: usize,
    },

    /// A caller-supplied parameter was invalid; the message describes why.
    #[error("{0}")]
    InvalidParameter(String),
}

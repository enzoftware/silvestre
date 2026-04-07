use crate::{Result, SilvestreError, SilvestreImage};

/// Strategy for sampling pixels that fall outside the image bounds during
/// convolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderMode {
    /// Treat out-of-bounds pixels as zero (zero-padding).
    Zero,
    /// Repeat the value of the nearest edge pixel.
    Clamp,
    /// Reflect coordinates across the edge without repeating the border
    /// pixel: `... 2 1 0 1 2 ... n-2 n-1 n-2 ...`.
    Mirror,
}

/// A 2D convolution kernel with odd width and height.
#[derive(Debug, Clone)]
pub struct Kernel {
    width: usize,
    height: usize,
    values: Vec<f32>,
}

impl Kernel {
    /// Create a new kernel from a row-major value buffer.
    ///
    /// `width` and `height` must be positive odd integers, and
    /// `values.len()` must equal `width * height`.
    pub fn new(values: Vec<f32>, width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(SilvestreError::InvalidParameter(
                "kernel dimensions must be greater than zero".into(),
            ));
        }
        if width.is_multiple_of(2) || height.is_multiple_of(2) {
            return Err(SilvestreError::InvalidParameter(
                "kernel dimensions must be odd".into(),
            ));
        }
        let expected = width
            .checked_mul(height)
            .ok_or_else(|| SilvestreError::InvalidParameter("kernel size overflow".into()))?;
        if values.len() != expected {
            return Err(SilvestreError::BufferSizeMismatch {
                expected,
                got: values.len(),
            });
        }
        Ok(Self {
            width,
            height,
            values,
        })
    }

    /// Create a new kernel by copying values from a slice. Useful when the
    /// caller already owns a `&[f32]` and wants to avoid cloning into a
    /// `Vec` at the call site.
    pub fn from_slice(values: &[f32], width: usize, height: usize) -> Result<Self> {
        Self::new(values.to_vec(), width, height)
    }

    /// Convenience constructor for a square kernel of side `size`.
    pub fn square(values: Vec<f32>, size: usize) -> Result<Self> {
        Self::new(values, size, size)
    }

    /// Build an identity kernel of the given odd size. Convolving any image
    /// with this kernel returns an image equal to the input.
    pub fn identity(size: usize) -> Result<Self> {
        if size == 0 || size.is_multiple_of(2) {
            return Err(SilvestreError::InvalidParameter(
                "kernel size must be a positive odd integer".into(),
            ));
        }
        let mut values = vec![0.0_f32; size * size];
        let center = size / 2;
        values[center * size + center] = 1.0;
        Self::square(values, size)
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    #[must_use]
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

/// A separable 2D kernel expressed as the outer product of a horizontal and
/// vertical 1D vector. Applying it via [`apply_separable_kernel`] is
/// `O(N * (kw + kh))` instead of the `O(N * kw * kh)` cost of a full 2D
/// convolution.
#[derive(Debug, Clone)]
pub struct SeparableKernel {
    horizontal: Vec<f32>,
    vertical: Vec<f32>,
}

impl SeparableKernel {
    /// Create a new separable kernel. Both components must have positive odd
    /// length.
    pub fn new(horizontal: Vec<f32>, vertical: Vec<f32>) -> Result<Self> {
        if horizontal.is_empty() || vertical.is_empty() {
            return Err(SilvestreError::InvalidParameter(
                "separable kernel components must be non-empty".into(),
            ));
        }
        if horizontal.len().is_multiple_of(2) || vertical.len().is_multiple_of(2) {
            return Err(SilvestreError::InvalidParameter(
                "separable kernel components must have odd length".into(),
            ));
        }
        Ok(Self {
            horizontal,
            vertical,
        })
    }

    #[must_use]
    pub fn horizontal(&self) -> &[f32] {
        &self.horizontal
    }

    #[must_use]
    pub fn vertical(&self) -> &[f32] {
        &self.vertical
    }
}

/// Apply a 2D convolution kernel to `image`, returning a new image of the
/// same dimensions and color space.
///
/// The kernel is applied independently to every channel (including alpha).
/// Out-of-bounds samples are resolved according to `border`. Accumulated
/// values are rounded and clamped to the `0..=255` range.
pub fn apply_kernel(
    image: &SilvestreImage,
    kernel: &Kernel,
    border: BorderMode,
) -> Result<SilvestreImage> {
    let width = image.width();
    let height = image.height();
    let channels = image.color_space().channels();
    let src = image.pixels();
    let mut dst = vec![0u8; src.len()];

    if width == 0 || height == 0 {
        return SilvestreImage::new(dst, width, height, image.color_space());
    }

    let stride = (width as usize) * channels;
    let kw = kernel.width as i64;
    let kh = kernel.height as i64;
    let kw_half = kw / 2;
    let kh_half = kh / 2;
    let kernel_values = kernel.values.as_slice();

    for y in 0..height as i64 {
        for x in 0..width as i64 {
            for c in 0..channels {
                let mut acc = 0.0_f32;
                for ky in 0..kh {
                    for kx in 0..kw {
                        let sx = x + kx - kw_half;
                        let sy = y + ky - kh_half;
                        let sample =
                            sample(src, width, height, channels, stride, sx, sy, c, border);
                        let weight = kernel_values[(ky * kw + kx) as usize];
                        acc += f32::from(sample) * weight;
                    }
                }
                let idx = (y as usize) * stride + (x as usize) * channels + c;
                dst[idx] = acc.round().clamp(0.0, 255.0) as u8;
            }
        }
    }

    SilvestreImage::new(dst, width, height, image.color_space())
}

/// Apply a separable kernel by performing a horizontal pass followed by a
/// vertical pass. Equivalent to a full 2D convolution with the outer product
/// of the two components, but cheaper for kernels of meaningful size.
pub fn apply_separable_kernel(
    image: &SilvestreImage,
    kernel: &SeparableKernel,
    border: BorderMode,
) -> Result<SilvestreImage> {
    let horizontal = Kernel::from_slice(&kernel.horizontal, kernel.horizontal.len(), 1)?;
    let vertical = Kernel::from_slice(&kernel.vertical, 1, kernel.vertical.len())?;
    let intermediate = apply_kernel(image, &horizontal, border)?;
    apply_kernel(&intermediate, &vertical, border)
}

#[allow(clippy::too_many_arguments)]
fn sample(
    src: &[u8],
    width: u32,
    height: u32,
    channels: usize,
    stride: usize,
    x: i64,
    y: i64,
    c: usize,
    border: BorderMode,
) -> u8 {
    let w = width as i64;
    let h = height as i64;
    let (sx, sy) = match border {
        BorderMode::Zero => {
            if x < 0 || x >= w || y < 0 || y >= h {
                return 0;
            }
            (x, y)
        }
        BorderMode::Clamp => (x.clamp(0, w - 1), y.clamp(0, h - 1)),
        BorderMode::Mirror => (mirror(x, w), mirror(y, h)),
    };
    src[sy as usize * stride + sx as usize * channels + c]
}

fn mirror(coord: i64, size: i64) -> i64 {
    debug_assert!(size > 0);
    if size == 1 {
        return 0;
    }
    let period = 2 * (size - 1);
    let mut m = coord.rem_euclid(period);
    if m >= size {
        m = period - m;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn kernel_new_rejects_even_dimensions() {
        let err = Kernel::new(vec![0.0; 4], 2, 2).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn kernel_new_rejects_zero_dimensions() {
        let err = Kernel::new(vec![], 0, 1).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn kernel_new_rejects_mismatched_buffer() {
        let err = Kernel::new(vec![0.0; 8], 3, 3).unwrap_err();
        assert!(matches!(
            err,
            SilvestreError::BufferSizeMismatch {
                expected: 9,
                got: 8
            }
        ));
    }

    #[test]
    fn identity_kernel_returns_same_image() {
        let img = gray_image(3, 3, vec![10, 20, 30, 40, 50, 60, 70, 80, 90]);
        let kernel = Kernel::identity(3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn identity_kernel_size_one() {
        let img = gray_image(2, 2, vec![1, 2, 3, 4]);
        let kernel = Kernel::identity(1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn box_blur_3x3_clamp() {
        // 3x3 box blur over a uniform image yields the same uniform image
        // regardless of border handling.
        let img = gray_image(4, 4, vec![100; 16]);
        let values = vec![1.0_f32 / 9.0; 9];
        let kernel = Kernel::square(values, 3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 100));
    }

    #[test]
    fn zero_border_darkens_edges() {
        // 3x3 box blur on a uniform image with zero padding should produce
        // dimmer values on the border (since padded zeros pull the average
        // down).
        let img = gray_image(3, 3, vec![90; 9]);
        let values = vec![1.0_f32 / 9.0; 9];
        let kernel = Kernel::square(values, 3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        // center pixel (row 1, col 1 -> index 4) sees no padding
        assert_eq!(out.pixels()[4], 90);
        // corner sees 5 of 9 cells as padding
        assert!(out.pixels()[0] < 90);
    }

    #[test]
    fn rgba_alpha_channel_is_processed() {
        let img = SilvestreImage::new(
            vec![10, 20, 30, 255, 40, 50, 60, 200],
            2,
            1,
            ColorSpace::Rgba,
        )
        .unwrap();
        let kernel = Kernel::identity(3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn convolution_preserves_dimensions_and_color_space() {
        let img = gray_image(5, 4, (0..20u8).collect());
        let kernel = Kernel::identity(3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Mirror).unwrap();
        assert_eq!(out.width(), 5);
        assert_eq!(out.height(), 4);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn negative_weights_clamped_to_zero() {
        // A kernel that subtracts the center pixel from itself should produce
        // negative values that get clamped to zero.
        let img = gray_image(1, 1, vec![100]);
        let kernel = Kernel::new(vec![-1.0], 1, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        assert_eq!(out.pixels(), &[0]);
    }

    #[test]
    fn large_weights_clamped_to_max() {
        let img = gray_image(1, 1, vec![100]);
        let kernel = Kernel::new(vec![10.0], 1, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        assert_eq!(out.pixels(), &[255]);
    }

    #[test]
    fn empty_image_yields_empty_image() {
        let img = gray_image(0, 0, vec![]);
        let kernel = Kernel::identity(3).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        assert_eq!(out.width(), 0);
        assert_eq!(out.height(), 0);
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn rectangular_kernel_horizontal_blur() {
        let img = gray_image(5, 1, vec![0, 0, 100, 0, 0]);
        let kernel = Kernel::new(vec![1.0 / 3.0; 3], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        // The "100" should be spread to its neighbours.
        let p = out.pixels();
        assert_eq!(p[0], 0);
        assert_eq!(p[1], 33);
        assert_eq!(p[2], 33);
        assert_eq!(p[3], 33);
        assert_eq!(p[4], 0);
    }

    #[test]
    fn separable_matches_full_convolution() {
        // Box blur 3x3 = horizontal [1/3,1/3,1/3] outer vertical [1/3,1/3,1/3]
        let img = gray_image(4, 4, (0..16u8).collect());
        let full = Kernel::square(vec![1.0_f32 / 9.0; 9], 3).unwrap();
        let separable =
            SeparableKernel::new(vec![1.0_f32 / 3.0; 3], vec![1.0_f32 / 3.0; 3]).unwrap();
        let full_out = apply_kernel(&img, &full, BorderMode::Clamp).unwrap();
        let sep_out = apply_separable_kernel(&img, &separable, BorderMode::Clamp).unwrap();
        // Allow off-by-one rounding differences from the two-pass rounding.
        for (a, b) in full_out.pixels().iter().zip(sep_out.pixels().iter()) {
            assert!((i32::from(*a) - i32::from(*b)).abs() <= 1, "{a} vs {b}");
        }
    }

    #[test]
    fn separable_kernel_rejects_even_components() {
        let err = SeparableKernel::new(vec![0.0; 2], vec![0.0; 3]).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn separable_kernel_rejects_empty_components() {
        let err = SeparableKernel::new(vec![], vec![1.0]).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn mirror_helper_reflects_coordinates() {
        // size = 4: indices 0,1,2,3 -> period 6
        // -1 -> 1, -2 -> 2, 4 -> 2, 5 -> 1, 6 -> 0
        assert_eq!(mirror(-1, 4), 1);
        assert_eq!(mirror(-2, 4), 2);
        assert_eq!(mirror(0, 4), 0);
        assert_eq!(mirror(3, 4), 3);
        assert_eq!(mirror(4, 4), 2);
        assert_eq!(mirror(5, 4), 1);
        assert_eq!(mirror(6, 4), 0);
    }

    #[test]
    fn mirror_size_one() {
        assert_eq!(mirror(-3, 1), 0);
        assert_eq!(mirror(0, 1), 0);
        assert_eq!(mirror(7, 1), 0);
    }

    #[test]
    fn clamp_border_uses_edge_values() {
        let img = gray_image(3, 1, vec![10, 20, 30]);
        // Horizontal kernel that picks the pixel one to the left.
        let kernel = Kernel::new(vec![1.0, 0.0, 0.0], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        // x=0: samples x=-1 which clamps to x=0 -> 10
        // x=1: samples x=0 -> 10
        // x=2: samples x=1 -> 20
        assert_eq!(out.pixels(), &[10, 10, 20]);
    }

    #[test]
    fn mirror_border_reflects_values() {
        let img = gray_image(3, 1, vec![10, 20, 30]);
        let kernel = Kernel::new(vec![1.0, 0.0, 0.0], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Mirror).unwrap();
        // x=0 samples x=-1 -> mirrored to x=1 -> 20
        assert_eq!(out.pixels(), &[20, 10, 20]);
    }
}

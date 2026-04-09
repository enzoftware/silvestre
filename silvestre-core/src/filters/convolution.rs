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
        let expected: usize = width
            .checked_mul(height)
            .ok_or_else(|| SilvestreError::InvalidParameter("kernel size overflow".into()))?;
        if values.len() != expected {
            return Err(SilvestreError::InvalidParameter(format!(
                "kernel value count mismatch: expected {expected}, got {}",
                values.len()
            )));
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
        let len = size
            .checked_mul(size)
            .ok_or_else(|| SilvestreError::InvalidParameter("kernel size overflow".into()))?;
        let mut values = vec![0.0_f32; len];
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

    if width == 0 || height == 0 {
        return SilvestreImage::new(
            vec![0u8; image.pixels().len()],
            width,
            height,
            image.color_space(),
        );
    }

    let src_f32: Vec<f32> = image.pixels().iter().map(|&b| f32::from(b)).collect();
    let dst_f32 = convolve_f32(&src_f32, width, height, channels, kernel, border);
    let dst = quantize(&dst_f32);
    SilvestreImage::new(dst, width, height, image.color_space())
}

/// Apply a separable kernel by performing a horizontal pass followed by a
/// vertical pass. Equivalent to a full 2D convolution with the outer product
/// of the two components, but cheaper for kernels of meaningful size. The
/// intermediate pass is kept in `f32` so that kernels with negative or
/// out-of-range intermediate responses (e.g. Sobel) match the full 2D
/// convolution result.
pub fn apply_separable_kernel(
    image: &SilvestreImage,
    kernel: &SeparableKernel,
    border: BorderMode,
) -> Result<SilvestreImage> {
    let width = image.width();
    let height = image.height();
    let channels = image.color_space().channels();

    if width == 0 || height == 0 {
        return SilvestreImage::new(
            vec![0u8; image.pixels().len()],
            width,
            height,
            image.color_space(),
        );
    }

    let horizontal = Kernel::from_slice(&kernel.horizontal, kernel.horizontal.len(), 1)?;
    let vertical = Kernel::from_slice(&kernel.vertical, 1, kernel.vertical.len())?;

    let src_f32: Vec<f32> = image.pixels().iter().map(|&b| f32::from(b)).collect();
    let after_h = convolve_f32(&src_f32, width, height, channels, &horizontal, border);
    let after_v = convolve_f32(&after_h, width, height, channels, &vertical, border);
    let dst = quantize(&after_v);
    SilvestreImage::new(dst, width, height, image.color_space())
}

fn convolve_f32(
    src: &[f32],
    width: u32,
    height: u32,
    channels: usize,
    kernel: &Kernel,
    border: BorderMode,
) -> Vec<f32> {
    let mut dst = vec![0.0_f32; src.len()];
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
                            sample_f32(src, width, height, channels, stride, sx, sy, c, border);
                        // Flip the kernel indices so that this is a true
                        // convolution (not correlation). This ensures that
                        // directional kernels behave consistently with the
                        // mathematical definition.
                        let weight = kernel_values[((kh - 1 - ky) * kw + (kw - 1 - kx)) as usize];
                        acc += sample * weight;
                    }
                }
                let idx = (y as usize) * stride + (x as usize) * channels + c;
                dst[idx] = acc;
            }
        }
    }

    dst
}

fn quantize(src: &[f32]) -> Vec<u8> {
    src.iter()
        .map(|&v| v.round().clamp(0.0, 255.0) as u8)
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn sample_f32(
    src: &[f32],
    width: u32,
    height: u32,
    channels: usize,
    stride: usize,
    x: i64,
    y: i64,
    c: usize,
    border: BorderMode,
) -> f32 {
    let w = width as i64;
    let h = height as i64;
    let (sx, sy) = match border {
        BorderMode::Zero => {
            if x < 0 || x >= w || y < 0 || y >= h {
                return 0.0;
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
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
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
        // Horizontal kernel that (after the convolution flip) picks the
        // pixel one to the left. Stored in unflipped form as [0, 0, 1].
        let kernel = Kernel::new(vec![0.0, 0.0, 1.0], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Clamp).unwrap();
        // x=0: samples x=-1 which clamps to x=0 -> 10
        // x=1: samples x=0 -> 10
        // x=2: samples x=1 -> 20
        assert_eq!(out.pixels(), &[10, 10, 20]);
    }

    #[test]
    fn mirror_border_reflects_values() {
        let img = gray_image(3, 1, vec![10, 20, 30]);
        let kernel = Kernel::new(vec![0.0, 0.0, 1.0], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Mirror).unwrap();
        // x=0 samples x=-1 -> mirrored to x=1 -> 20
        assert_eq!(out.pixels(), &[20, 10, 20]);
    }

    #[test]
    fn convolution_flips_asymmetric_kernel() {
        // An asymmetric horizontal kernel [1, 2, 3] in convolution must be
        // flipped before being applied, yielding an effective weight of
        // [3, 2, 1] read left-to-right.
        let img = gray_image(5, 1, vec![0, 0, 100, 0, 0]);
        let kernel = Kernel::new(vec![1.0, 2.0, 3.0], 3, 1).unwrap();
        let out = apply_kernel(&img, &kernel, BorderMode::Zero).unwrap();
        // x=1: 3*0 + 2*0 + 1*100 = 100
        // x=2: 3*0 + 2*100 + 1*0 = 200
        // x=3: 3*100 + 2*0 + 1*0 = 300 -> clamp 255
        assert_eq!(out.pixels(), &[0, 100, 200, 255, 0]);
    }

    #[test]
    fn separable_preserves_sign_between_passes() {
        // Use a non-monotonic image so that the horizontal Sobel pass
        // produces both positive and negative intermediate values. If the
        // intermediate buffer were quantized to u8, the negatives would be
        // clamped to zero and the vertical pass would produce results that
        // diverge from the equivalent full 2D convolution.
        let img = gray_image(
            5,
            3,
            vec![50, 10, 30, 10, 50, 10, 50, 30, 50, 10, 50, 10, 30, 10, 50],
        );
        // Sobel-x = [1,0,-1] * [1,2,1]^T (outer product).
        let full_values = vec![1.0, 0.0, -1.0, 2.0, 0.0, -2.0, 1.0, 0.0, -1.0];
        let full = Kernel::square(full_values, 3).unwrap();
        let separable = SeparableKernel::new(vec![1.0, 0.0, -1.0], vec![1.0, 2.0, 1.0]).unwrap();
        let full_out = apply_kernel(&img, &full, BorderMode::Clamp).unwrap();
        let sep_out = apply_separable_kernel(&img, &separable, BorderMode::Clamp).unwrap();
        assert_eq!(full_out.pixels(), sep_out.pixels());
    }
}

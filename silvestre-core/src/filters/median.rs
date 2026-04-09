//! Median filter.
//!
//! A non-linear spatial filter that replaces each pixel with the median of
//! its neighborhood. The median filter is particularly effective at
//! removing salt-and-pepper (impulse) noise while preserving edges better
//! than a linear blur.
//!
//! # Quick start
//!
//! ```
//! use silvestre_core::filters::{Filter, MedianFilter};
//! use silvestre_core::{ColorSpace, SilvestreImage};
//!
//! let img = SilvestreImage::new(vec![100; 16], 4, 4, ColorSpace::Grayscale)?;
//! let filter = MedianFilter::new(3)?;
//! let out = filter.apply(&img)?;
//! assert_eq!(out.pixels(), img.pixels());
//! # Ok::<_, silvestre_core::SilvestreError>(())
//! ```

use crate::filters::convolution::{resolve_coord, BorderMode};
use crate::filters::Filter;
use crate::{Result, SilvestreError, SilvestreImage};

/// Non-linear median filter with a configurable square window.
///
/// Each output pixel is the per-channel median of a `size x size`
/// neighborhood centered on the input pixel. Out-of-bounds samples are
/// resolved according to the configured [`BorderMode`]; by default the
/// filter uses [`BorderMode::Clamp`], which matches the behavior of the
/// original Java reference implementation (which simply skipped the
/// border).
///
/// # Examples
///
/// Build a 3x3 median filter and apply it to a noisy grayscale image:
///
/// ```
/// use silvestre_core::filters::{Filter, MedianFilter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let pixels = vec![
///     100, 100, 100, 100,
///     100, 255, 100, 100, // salt
///     100, 100,   0, 100, // pepper
///     100, 100, 100, 100,
/// ];
/// let img = SilvestreImage::new(pixels, 4, 4, ColorSpace::Grayscale)?;
/// let filter = MedianFilter::new(3)?;
/// let out = filter.apply(&img)?;
/// // Salt and pepper are suppressed: the filtered center region is all 100s.
/// assert_eq!(out.pixels()[5], 100);
/// assert_eq!(out.pixels()[10], 100);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MedianFilter {
    size: usize,
    border: BorderMode,
}

impl MedianFilter {
    /// Create a new median filter with the given odd window size.
    ///
    /// Uses [`BorderMode::Clamp`] for edge handling. Returns an error if
    /// `size` is zero or even.
    pub fn new(size: usize) -> Result<Self> {
        Self::with_border(size, BorderMode::Clamp)
    }

    /// Create a new median filter with an explicit border mode.
    ///
    /// Returns an error if `size` is zero or even.
    pub fn with_border(size: usize, border: BorderMode) -> Result<Self> {
        if size == 0 || size.is_multiple_of(2) {
            return Err(SilvestreError::InvalidParameter(
                "median filter size must be a positive odd integer".into(),
            ));
        }
        Ok(Self { size, border })
    }

    /// Window side length (always an odd integer).
    #[must_use]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Border handling strategy for out-of-bounds samples.
    #[must_use]
    pub fn border(&self) -> BorderMode {
        self.border
    }
}

impl Default for MedianFilter {
    /// Default 3x3 median filter with [`BorderMode::Clamp`].
    fn default() -> Self {
        Self {
            size: 3,
            border: BorderMode::Clamp,
        }
    }
}

impl Filter for MedianFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
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

        let src = image.pixels();
        let stride = (width as usize) * channels;
        let mut dst = vec![0u8; src.len()];

        let half = (self.size / 2) as i64;
        // Window area is bounded by typical kernel sizes; allocate once and
        // reuse for every output pixel/channel.
        let window_area = self.size * self.size;
        let mut window: Vec<u8> = Vec::with_capacity(window_area);

        for y in 0..height as i64 {
            for x in 0..width as i64 {
                for c in 0..channels {
                    window.clear();
                    for ky in -half..=half {
                        for kx in -half..=half {
                            let sample = match resolve_coord(
                                x + kx,
                                y + ky,
                                width,
                                height,
                                self.border,
                            ) {
                                Some((sx, sy)) => src[sy * stride + sx * channels + c],
                                None => 0,
                            };
                            window.push(sample);
                        }
                    }
                    // Partial sort: we only need the median element.
                    let mid = window.len() / 2;
                    let (_, median, _) = window.select_nth_unstable(mid);
                    let median_value = *median;
                    let idx = (y as usize) * stride + (x as usize) * channels + c;
                    dst[idx] = median_value;
                }
            }
        }

        SilvestreImage::new(dst, width, height, image.color_space())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn rejects_even_size() {
        let err = MedianFilter::new(2).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_zero_size() {
        let err = MedianFilter::new(0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn accepts_odd_sizes() {
        assert!(MedianFilter::new(1).is_ok());
        assert!(MedianFilter::new(3).is_ok());
        assert!(MedianFilter::new(5).is_ok());
        assert!(MedianFilter::new(7).is_ok());
    }

    #[test]
    fn default_is_3x3_clamp() {
        let f = MedianFilter::default();
        assert_eq!(f.size(), 3);
        assert_eq!(f.border(), BorderMode::Clamp);
    }

    #[test]
    fn identity_on_uniform_image() {
        let img = gray_image(5, 5, vec![42; 25]);
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn size_one_is_identity() {
        let img = gray_image(3, 3, vec![10, 20, 30, 40, 50, 60, 70, 80, 90]);
        let filter = MedianFilter::new(1).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels(), img.pixels());
    }

    #[test]
    fn removes_salt_and_pepper_noise() {
        // 5x5 uniform image with a few salt (255) and pepper (0) impulses.
        // A 3x3 median filter should fully suppress isolated impulses.
        let mut pixels = vec![128u8; 25];
        pixels[6] = 255; // salt at (1, 1)
        pixels[12] = 0; // pepper at (2, 2)
        pixels[18] = 255; // salt at (3, 3)
        let img = gray_image(5, 5, pixels);
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        // All interior pixels should be restored to 128.
        for y in 1..4 {
            for x in 1..4 {
                assert_eq!(out.pixels()[y * 5 + x], 128, "pixel ({x}, {y})");
            }
        }
    }

    #[test]
    fn computes_median_of_sorted_window() {
        // 3x3 image: median of [10,20,30,40,50,60,70,80,90] = 50.
        let img = gray_image(3, 3, vec![10, 20, 30, 40, 50, 60, 70, 80, 90]);
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        // Center pixel sees the full window, so its output must be 50.
        assert_eq!(out.pixels()[4], 50);
    }

    #[test]
    fn processes_rgba_channels_independently() {
        // 3x3 RGBA image. The red channel has a salt impulse at the center,
        // the green channel has a pepper, and blue is uniform. The alpha
        // channel is also uniform. A 3x3 median must remove both impulses.
        let pixels = vec![
            100, 100, 50, 255, 100, 100, 50, 255, 100, 100, 50, 255, // row 0
            100, 100, 50, 255, 255, 0, 50, 255, 100, 100, 50, 255, // row 1
            100, 100, 50, 255, 100, 100, 50, 255, 100, 100, 50, 255, // row 2
        ];
        let img = SilvestreImage::new(pixels, 3, 3, ColorSpace::Rgba).unwrap();
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        // Center pixel channels should be restored to their neighborhood
        // medians: R=100, G=100, B=50, A=255.
        let center = &out.pixels()[4 * 4..4 * 4 + 4];
        assert_eq!(center, &[100, 100, 50, 255]);
    }

    #[test]
    fn preserves_dimensions_and_color_space() {
        let img = gray_image(5, 4, (0..20u8).collect());
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 5);
        assert_eq!(out.height(), 4);
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn empty_image_yields_empty_image() {
        let img = gray_image(0, 0, vec![]);
        let filter = MedianFilter::new(3).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 0);
        assert_eq!(out.height(), 0);
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn clamp_border_handles_edges() {
        // 3x1 image; at x=0 the 3x3 window sees [a,a,a, a,a,b, a,a,b] when
        // clamped. The median of that set should match the nearest edge.
        let img = gray_image(3, 1, vec![10, 20, 30]);
        let filter = MedianFilter::with_border(3, BorderMode::Clamp).unwrap();
        let out = filter.apply(&img).unwrap();
        // With clamping, the leftmost pixel's window is dominated by 10s,
        // so the median is 10.
        assert_eq!(out.pixels()[0], 10);
        // Center is the median of [10,20,30] = 20.
        assert_eq!(out.pixels()[1], 20);
        assert_eq!(out.pixels()[2], 30);
    }

    #[test]
    fn zero_border_darkens_edges() {
        // 3x3 uniform bright image; with zero padding the corner pixel's
        // 3x3 window contains 5 zeros and 4 values of 200, so the median
        // (5th of 9 sorted) is 0.
        let img = gray_image(3, 3, vec![200; 9]);
        let filter = MedianFilter::with_border(3, BorderMode::Zero).unwrap();
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels()[0], 0);
        // Center sees no padding -> median is 200.
        assert_eq!(out.pixels()[4], 200);
    }

    #[test]
    fn mirror_border_reflects_values() {
        // Uniform image -> any border mode yields the same uniform image.
        let img = gray_image(4, 4, vec![77; 16]);
        let filter = MedianFilter::with_border(5, BorderMode::Mirror).unwrap();
        let out = filter.apply(&img).unwrap();
        assert!(out.pixels().iter().all(|&v| v == 77));
    }

    #[test]
    fn filter_trait_object() {
        // Ensure MedianFilter can be used through a trait object.
        let filter: Box<dyn Filter> = Box::new(MedianFilter::new(3).unwrap());
        let img = gray_image(3, 3, vec![5; 9]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.pixels(), &[5; 9]);
    }
}

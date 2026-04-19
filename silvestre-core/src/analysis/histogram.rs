//! Per-channel histogram computation and basic intensity statistics.
//!
//! A histogram counts how many pixels have each possible intensity value
//! (0–255) for every channel in the image. Together with the derived
//! statistics it provides a compact summary of the tonal distribution of
//! an image.
//!
//! # Quick start
//!
//! ```
//! use silvestre_core::analysis::histogram::Histogram;
//! use silvestre_core::{ColorSpace, SilvestreImage};
//!
//! // Solid red 2×2 RGB image.
//! let img = SilvestreImage::new(vec![255, 0, 0].repeat(4), 2, 2, ColorSpace::Rgb)?;
//! let hist = Histogram::compute(&img);
//!
//! // Red channel: all 4 pixels have value 255.
//! assert_eq!(hist.channel(0)[255], 4);
//! // Green and blue channels: all pixels are 0.
//! assert_eq!(hist.channel(1)[0], 4);
//! assert_eq!(hist.channel(2)[0], 4);
//! # Ok::<_, silvestre_core::SilvestreError>(())
//! ```

use crate::{ColorSpace, SilvestreImage};

/// Number of intensity bins in a histogram (one per possible byte value).
pub const BINS: usize = 256;

/// Per-channel pixel intensity statistics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelStats {
    /// Minimum intensity value present in the channel.
    pub min: u8,
    /// Maximum intensity value present in the channel.
    pub max: u8,
    /// Arithmetic mean of pixel intensities.
    pub mean: f64,
    /// Population standard deviation of pixel intensities.
    pub std_dev: f64,
}

/// Per-channel histogram and derived statistics for a [`SilvestreImage`].
///
/// Each channel stores 256 bins counting how many pixels hold each value.
/// The number of channels matches [`ColorSpace::channels`] for the source
/// image. Statistics are computed once at construction and cached.
///
/// # Examples
///
/// ```
/// use silvestre_core::analysis::histogram::Histogram;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // All-black 3×3 grayscale image.
/// let img = SilvestreImage::new(vec![0; 9], 3, 3, ColorSpace::Grayscale)?;
/// let hist = Histogram::compute(&img);
///
/// assert_eq!(hist.num_channels(), 1);
/// assert_eq!(hist.channel(0)[0], 9);   // all 9 pixels are intensity 0
/// assert_eq!(hist.stats(0).min, 0);
/// assert_eq!(hist.stats(0).max, 0);
/// assert!((hist.stats(0).mean - 0.0).abs() < 1e-9);
/// assert!((hist.stats(0).std_dev - 0.0).abs() < 1e-9);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Histogram {
    /// `bins[c][v]` = number of pixels with value `v` in channel `c`.
    bins: Vec<[u64; BINS]>,
    /// Cached per-channel statistics.
    stats: Vec<ChannelStats>,
    /// Number of pixels that were counted.
    pixel_count: u64,
}

impl Histogram {
    /// Compute the histogram for every channel of `image`.
    ///
    /// Iterates the raw pixel buffer once, accumulating counts and the sums
    /// needed for mean/stddev in a single pass.
    #[must_use]
    pub fn compute(image: &SilvestreImage) -> Self {
        let channels = image.color_space().channels();
        let src = image.pixels();
        let pixel_count = (image.width() as u64) * (image.height() as u64);

        let mut bins = vec![[0u64; BINS]; channels];
        // Accumulators for mean and variance (two-pass: first accumulate
        // counts/sums, then derive statistics per channel).
        let mut sums = vec![0u64; channels];

        for chunk in src.chunks_exact(channels) {
            for (c, &val) in chunk.iter().enumerate() {
                bins[c][val as usize] += 1;
                sums[c] += val as u64;
            }
        }

        let stats = if pixel_count == 0 {
            vec![ChannelStats::zero(); channels]
        } else {
            (0..channels)
                .map(|c| compute_channel_stats(&bins[c], pixel_count, sums[c]))
                .collect()
        };

        Self {
            bins,
            stats,
            pixel_count,
        }
    }

    /// The raw 256-bin count array for channel `c`.
    ///
    /// # Panics
    ///
    /// Panics if `c >= self.num_channels()`.
    #[must_use]
    pub fn channel(&self, c: usize) -> &[u64; BINS] {
        &self.bins[c]
    }

    /// Derived statistics for channel `c`.
    ///
    /// # Panics
    ///
    /// Panics if `c >= self.num_channels()`.
    #[must_use]
    pub fn stats(&self, c: usize) -> &ChannelStats {
        &self.stats[c]
    }

    /// Number of channels (matches `image.color_space().channels()`).
    #[must_use]
    pub fn num_channels(&self) -> usize {
        self.bins.len()
    }

    /// Total number of pixels that were counted.
    #[must_use]
    pub fn pixel_count(&self) -> u64 {
        self.pixel_count
    }

    /// Compute a single-channel luminance histogram from an RGB or RGBA image
    /// using ITU-R BT.601 weights: `Y = 0.299·R + 0.587·G + 0.114·B`.
    ///
    /// For a [`ColorSpace::Grayscale`] image this is identical to
    /// [`Histogram::compute`].
    ///
    /// Returns `None` if the image has an unsupported color space (currently
    /// never happens with the three defined variants, but kept for forward
    /// compatibility).
    ///
    /// # Examples
    ///
    /// ```
    /// use silvestre_core::analysis::histogram::Histogram;
    /// use silvestre_core::{ColorSpace, SilvestreImage};
    ///
    /// // Pure white 1×1 RGB image → luminance = 255.
    /// let img = SilvestreImage::new(vec![255, 255, 255], 1, 1, ColorSpace::Rgb)?;
    /// let lum = Histogram::luminance(&img).unwrap();
    /// assert_eq!(lum.channel(0)[255], 1);
    /// # Ok::<_, silvestre_core::SilvestreError>(())
    /// ```
    #[must_use]
    pub fn luminance(image: &SilvestreImage) -> Option<Self> {
        let src = image.pixels();
        let pixel_count = (image.width() as u64) * (image.height() as u64);
        let mut bins = [0u64; BINS];
        let mut sum = 0u64;

        match image.color_space() {
            ColorSpace::Grayscale => {
                for &v in src {
                    bins[v as usize] += 1;
                    sum += v as u64;
                }
            }
            ColorSpace::Rgb => {
                for chunk in src.chunks_exact(3) {
                    let lum = luminance_bt601(chunk[0], chunk[1], chunk[2]);
                    bins[lum as usize] += 1;
                    sum += lum as u64;
                }
            }
            ColorSpace::Rgba => {
                for chunk in src.chunks_exact(4) {
                    let lum = luminance_bt601(chunk[0], chunk[1], chunk[2]);
                    bins[lum as usize] += 1;
                    sum += lum as u64;
                }
            }
            _ => return None,
        }

        let stats = if pixel_count == 0 {
            ChannelStats::zero()
        } else {
            compute_channel_stats(&bins, pixel_count, sum)
        };

        Some(Self {
            bins: vec![bins],
            stats: vec![stats],
            pixel_count,
        })
    }
}

impl ChannelStats {
    /// Construct a zero-valued stats struct (used for empty images).
    fn zero() -> Self {
        Self {
            min: 0,
            max: 0,
            mean: 0.0,
            std_dev: 0.0,
        }
    }
}

/// Derive [`ChannelStats`] from a filled bin array and its precomputed sum.
///
/// `pixel_count` must be > 0.
fn compute_channel_stats(bins: &[u64; BINS], pixel_count: u64, sum: u64) -> ChannelStats {
    let mean = sum as f64 / pixel_count as f64;

    let min = bins
        .iter()
        .enumerate()
        .find(|(_, &count)| count > 0)
        .map(|(v, _)| v as u8)
        .unwrap_or(0);

    let max = bins
        .iter()
        .enumerate()
        .rev()
        .find(|(_, &count)| count > 0)
        .map(|(v, _)| v as u8)
        .unwrap_or(0);

    let variance = bins
        .iter()
        .enumerate()
        .filter(|(_, &count)| count > 0)
        .map(|(v, &count)| {
            let diff = v as f64 - mean;
            diff * diff * count as f64
        })
        .sum::<f64>()
        / pixel_count as f64;

    ChannelStats {
        min,
        max,
        mean,
        std_dev: variance.sqrt(),
    }
}

/// ITU-R BT.601 luma from 8-bit RGB components.
#[inline]
fn luminance_bt601(r: u8, g: u8, b: u8) -> u8 {
    let lum = 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
    lum.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorSpace, SilvestreImage};

    fn gray(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    // ------------------------------------------------------------------ //
    // Basic construction
    // ------------------------------------------------------------------ //

    #[test]
    fn grayscale_single_channel() {
        let img = gray(2, 2, vec![0, 128, 255, 128]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.num_channels(), 1);
        assert_eq!(hist.pixel_count(), 4);
    }

    #[test]
    fn rgb_three_channels() {
        let img = SilvestreImage::new(vec![0; 3 * 4], 2, 2, ColorSpace::Rgb).unwrap();
        let hist = Histogram::compute(&img);
        assert_eq!(hist.num_channels(), 3);
    }

    #[test]
    fn rgba_four_channels() {
        let img = SilvestreImage::new(vec![0; 4 * 4], 2, 2, ColorSpace::Rgba).unwrap();
        let hist = Histogram::compute(&img);
        assert_eq!(hist.num_channels(), 4);
    }

    // ------------------------------------------------------------------ //
    // Bin counting
    // ------------------------------------------------------------------ //

    #[test]
    fn all_black_grayscale_spike_at_zero() {
        let img = gray(3, 3, vec![0; 9]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.channel(0)[0], 9);
        // Every other bin must be zero.
        assert!(hist.channel(0)[1..].iter().all(|&c| c == 0));
    }

    #[test]
    fn all_white_grayscale_spike_at_255() {
        let img = gray(4, 4, vec![255; 16]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.channel(0)[255], 16);
        assert!(hist.channel(0)[..255].iter().all(|&c| c == 0));
    }

    #[test]
    fn solid_color_rgb_single_spike_per_channel() {
        // Solid red 2×2 image.
        let pixels: Vec<u8> = vec![255, 0, 0].repeat(4);
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgb).unwrap();
        let hist = Histogram::compute(&img);
        // Red channel spike at 255.
        assert_eq!(hist.channel(0)[255], 4);
        assert!(hist.channel(0)[..255].iter().all(|&c| c == 0));
        // Green and blue spikes at 0.
        assert_eq!(hist.channel(1)[0], 4);
        assert!(hist.channel(1)[1..].iter().all(|&c| c == 0));
        assert_eq!(hist.channel(2)[0], 4);
        assert!(hist.channel(2)[1..].iter().all(|&c| c == 0));
    }

    #[test]
    fn bin_counts_sum_to_pixel_count() {
        let img = gray(5, 4, (0..20u8).collect());
        let hist = Histogram::compute(&img);
        let total: u64 = hist.channel(0).iter().sum();
        assert_eq!(total, hist.pixel_count());
    }

    #[test]
    fn bin_counts_sum_to_pixel_count_rgb() {
        let pixels: Vec<u8> = (0..48u8).collect();
        let img = SilvestreImage::new(pixels, 4, 4, ColorSpace::Rgb).unwrap();
        let hist = Histogram::compute(&img);
        for c in 0..3 {
            let total: u64 = hist.channel(c).iter().sum();
            assert_eq!(total, hist.pixel_count(), "channel {c}");
        }
    }

    #[test]
    fn known_distribution_counts_correctly() {
        // 4 pixels: values 10, 20, 20, 30
        let img = gray(4, 1, vec![10, 20, 20, 30]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.channel(0)[10], 1);
        assert_eq!(hist.channel(0)[20], 2);
        assert_eq!(hist.channel(0)[30], 1);
        // All other bins are zero.
        for v in 0..=255usize {
            if v != 10 && v != 20 && v != 30 {
                assert_eq!(hist.channel(0)[v], 0, "bin {v}");
            }
        }
    }

    // ------------------------------------------------------------------ //
    // Statistics — min / max
    // ------------------------------------------------------------------ //

    #[test]
    fn stats_min_max_uniform() {
        let img = gray(3, 3, vec![42; 9]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.stats(0).min, 42);
        assert_eq!(hist.stats(0).max, 42);
    }

    #[test]
    fn stats_min_max_range() {
        let img = gray(4, 1, vec![10, 50, 200, 77]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.stats(0).min, 10);
        assert_eq!(hist.stats(0).max, 200);
    }

    #[test]
    fn stats_min_zero_max_255() {
        let img = gray(2, 1, vec![0, 255]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.stats(0).min, 0);
        assert_eq!(hist.stats(0).max, 255);
    }

    // ------------------------------------------------------------------ //
    // Statistics — mean
    // ------------------------------------------------------------------ //

    #[test]
    fn stats_mean_uniform() {
        let img = gray(4, 4, vec![100; 16]);
        let hist = Histogram::compute(&img);
        let diff = (hist.stats(0).mean - 100.0).abs();
        assert!(diff < 1e-9, "mean={}", hist.stats(0).mean);
    }

    #[test]
    fn stats_mean_two_values() {
        // 2 pixels: 0 and 200 → mean = 100.
        let img = gray(2, 1, vec![0, 200]);
        let hist = Histogram::compute(&img);
        let diff = (hist.stats(0).mean - 100.0).abs();
        assert!(diff < 1e-9, "mean={}", hist.stats(0).mean);
    }

    #[test]
    fn stats_mean_known_sequence() {
        // values 0..=9 → mean = 4.5
        let img = gray(10, 1, (0..10u8).collect());
        let hist = Histogram::compute(&img);
        let diff = (hist.stats(0).mean - 4.5).abs();
        assert!(diff < 1e-9, "mean={}", hist.stats(0).mean);
    }

    // ------------------------------------------------------------------ //
    // Statistics — std_dev
    // ------------------------------------------------------------------ //

    #[test]
    fn stats_stddev_uniform_is_zero() {
        let img = gray(5, 5, vec![128; 25]);
        let hist = Histogram::compute(&img);
        assert!(hist.stats(0).std_dev < 1e-9, "std_dev={}", hist.stats(0).std_dev);
    }

    #[test]
    fn stats_stddev_two_symmetric_values() {
        // Values 0 and 200 → mean=100, variance = ((0-100)²+(200-100)²)/2 = 10000 → σ=100.
        let img = gray(2, 1, vec![0, 200]);
        let hist = Histogram::compute(&img);
        let diff = (hist.stats(0).std_dev - 100.0).abs();
        assert!(diff < 1e-6, "std_dev={}", hist.stats(0).std_dev);
    }

    #[test]
    fn stats_stddev_matches_manual_computation() {
        // values: 10, 20, 30 → mean=20, var=((−10)²+0²+10²)/3=200/3 → σ≈8.165
        let img = gray(3, 1, vec![10, 20, 30]);
        let hist = Histogram::compute(&img);
        let expected = (200.0f64 / 3.0).sqrt();
        let diff = (hist.stats(0).std_dev - expected).abs();
        assert!(diff < 1e-9, "std_dev={} expected={expected}", hist.stats(0).std_dev);
    }

    // ------------------------------------------------------------------ //
    // Multi-channel statistics
    // ------------------------------------------------------------------ //

    #[test]
    fn rgb_stats_per_channel_correct() {
        // 1×1 pixel: R=100, G=150, B=200
        let img = SilvestreImage::new(vec![100, 150, 200], 1, 1, ColorSpace::Rgb).unwrap();
        let hist = Histogram::compute(&img);
        assert_eq!(hist.stats(0).min, 100); // R
        assert_eq!(hist.stats(1).min, 150); // G
        assert_eq!(hist.stats(2).min, 200); // B
        assert_eq!(hist.stats(0).max, 100);
        assert_eq!(hist.stats(1).max, 150);
        assert_eq!(hist.stats(2).max, 200);
    }

    #[test]
    fn rgba_alpha_channel_counted_separately() {
        // 2 pixels: both fully opaque (A=255)
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255];
        let img = SilvestreImage::new(pixels, 2, 1, ColorSpace::Rgba).unwrap();
        let hist = Histogram::compute(&img);
        // Alpha channel (index 3): both pixels are 255.
        assert_eq!(hist.channel(3)[255], 2);
        assert_eq!(hist.stats(3).min, 255);
        assert_eq!(hist.stats(3).max, 255);
    }

    // ------------------------------------------------------------------ //
    // Luminance histogram
    // ------------------------------------------------------------------ //

    #[test]
    fn luminance_grayscale_identical_to_compute() {
        let img = gray(3, 3, (10..19u8).collect());
        let full = Histogram::compute(&img);
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.channel(0), full.channel(0));
        assert_eq!(lum.pixel_count(), full.pixel_count());
    }

    #[test]
    fn luminance_pure_white_rgb() {
        let img = SilvestreImage::new(vec![255, 255, 255], 1, 1, ColorSpace::Rgb).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.channel(0)[255], 1);
    }

    #[test]
    fn luminance_pure_black_rgb() {
        let img = SilvestreImage::new(vec![0, 0, 0], 1, 1, ColorSpace::Rgb).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.channel(0)[0], 1);
    }

    #[test]
    fn luminance_pure_red_rgb() {
        // Pure red → luma ≈ 76 (0.299*255 ≈ 76.245 → rounds to 76).
        let img = SilvestreImage::new(vec![255, 0, 0], 1, 1, ColorSpace::Rgb).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.channel(0)[76], 1);
    }

    #[test]
    fn luminance_rgba_ignores_alpha() {
        // Same color with different alphas → same luminance value.
        let pixels = vec![100, 200, 50, 255, 100, 200, 50, 0];
        let img = SilvestreImage::new(pixels, 2, 1, ColorSpace::Rgba).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.num_channels(), 1);
        // Both pixels map to the same luma value; total count = 2.
        assert_eq!(lum.pixel_count(), 2);
        let total: u64 = lum.channel(0).iter().sum();
        assert_eq!(total, 2);
    }

    #[test]
    fn luminance_returns_one_channel() {
        let img = SilvestreImage::new(vec![1, 2, 3, 4, 5, 6], 2, 1, ColorSpace::Rgb).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.num_channels(), 1);
    }

    // ------------------------------------------------------------------ //
    // Edge cases
    // ------------------------------------------------------------------ //

    #[test]
    fn empty_image_zero_stats() {
        let img = SilvestreImage::new(vec![], 0, 0, ColorSpace::Grayscale).unwrap();
        let hist = Histogram::compute(&img);
        assert_eq!(hist.pixel_count(), 0);
        assert_eq!(hist.num_channels(), 1);
        let s = hist.stats(0);
        assert_eq!(s.min, 0);
        assert_eq!(s.max, 0);
        assert!((s.mean - 0.0).abs() < 1e-9);
        assert!((s.std_dev - 0.0).abs() < 1e-9);
    }

    #[test]
    fn empty_image_luminance_zero_stats() {
        let img = SilvestreImage::new(vec![], 0, 0, ColorSpace::Rgb).unwrap();
        let lum = Histogram::luminance(&img).unwrap();
        assert_eq!(lum.pixel_count(), 0);
        let s = lum.stats(0);
        assert!((s.mean - 0.0).abs() < 1e-9);
    }

    #[test]
    fn single_pixel_stats() {
        let img = gray(1, 1, vec![77]);
        let hist = Histogram::compute(&img);
        assert_eq!(hist.pixel_count(), 1);
        assert_eq!(hist.stats(0).min, 77);
        assert_eq!(hist.stats(0).max, 77);
        let diff = (hist.stats(0).mean - 77.0).abs();
        assert!(diff < 1e-9);
        assert!(hist.stats(0).std_dev < 1e-9);
    }

    #[test]
    fn full_range_grayscale_all_bins_one() {
        // 256 pixels each with a distinct value 0..=255.
        let pixels: Vec<u8> = (0..=255u8).collect();
        let img = gray(256, 1, pixels);
        let hist = Histogram::compute(&img);
        assert!(hist.channel(0).iter().all(|&c| c == 1));
        assert_eq!(hist.stats(0).min, 0);
        assert_eq!(hist.stats(0).max, 255);
        // mean of 0..=255 = 127.5
        let diff = (hist.stats(0).mean - 127.5).abs();
        assert!(diff < 1e-6, "mean={}", hist.stats(0).mean);
    }
}

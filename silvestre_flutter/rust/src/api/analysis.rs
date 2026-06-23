use silvestre_core::analysis::histogram::Histogram;

use super::image::SilvestreImageWrapper;

/// Per-channel statistics returned from histogram computation.
pub struct ChannelStatsResult {
    pub min: u8,
    pub max: u8,
    pub mean: f64,
    pub std_dev: f64,
}

/// Result of a histogram computation.
///
/// `bins` is a `Vec<Vec<u64>>` where `bins[channel]` has 256 entries,
/// one count per possible intensity value (0–255).
pub struct HistogramResult {
    pub bins: Vec<Vec<u64>>,
    pub stats: Vec<ChannelStatsResult>,
    pub pixel_count: u64,
}

/// Compute a per-channel histogram for the image.
pub fn compute_histogram(img: &SilvestreImageWrapper) -> HistogramResult {
    let hist = Histogram::compute(&img.inner);
    histogram_to_result(&hist)
}

/// Compute a single-channel luminance histogram (BT.601 weights).
///
/// Returns `None` for unsupported color spaces (currently all are supported).
pub fn compute_luminance_histogram(img: &SilvestreImageWrapper) -> Option<HistogramResult> {
    Histogram::luminance(&img.inner).map(|h| histogram_to_result(&h))
}

fn histogram_to_result(hist: &Histogram) -> HistogramResult {
    let n = hist.num_channels();
    let mut bins = Vec::with_capacity(n);
    let mut stats = Vec::with_capacity(n);

    for c in 0..n {
        bins.push(hist.channel(c).to_vec());
        let s = hist.stats(c);
        stats.push(ChannelStatsResult {
            min: s.min,
            max: s.max,
            mean: s.mean,
            std_dev: s.std_dev,
        });
    }

    HistogramResult {
        bins,
        stats,
        pixel_count: hist.pixel_count(),
    }
}

//! Brick mosaic filter.
//!
//! Splits an image into a grid of `columns × rows` rectangular bricks and
//! paints each brick with a single solid color, producing a blocky mosaic
//! that can be rebuilt with physical bricks.
//!
//! The pipeline for every brick is:
//!
//! 1. **Flatten** — `Rgba` pixels are composited over an opaque background
//!    color (white by default), so the result never contains transparency.
//! 2. **Average** — the brick color is the rounded mean of its pixels.
//! 3. **Reduce** *(optional)* — when a color limit is set, the brick colors are
//!    reduced with median-cut and each brick snaps to the nearest palette
//!    color, so the [`BrickPlan::counts`] list stays short enough to shop for.
//!
//! Brick edges are placed at `floor(i · width / columns)` (and likewise for
//! rows), so the grid always covers the image exactly and brick sizes differ by
//! at most one pixel.

use std::collections::BTreeMap;

use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreError, SilvestreImage};

/// An opaque RGB color as `[r, g, b]`.
pub type BrickColor = [u8; 3];

/// Default background used to flatten transparent pixels (white).
pub const DEFAULT_BACKGROUND: BrickColor = [255, 255, 255];

/// Brick mosaic filter.
///
/// Divides the image into `columns × rows` bricks and fills each brick with its
/// average color. The output keeps the input's dimensions and color space;
/// `Rgba` output is fully opaque (alpha `255`).
///
/// Use [`BrickFilter::plan`] to get the per-brick layout and the number of
/// bricks needed of each color.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::brick::BrickFilter;
/// use silvestre_core::filters::Filter;
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// // 4×2 grayscale image split into 2×1 bricks.
/// let img = SilvestreImage::new(
///     vec![
///         0, 10, 200, 210,
///         20, 30, 220, 230,
///     ],
///     4, 2,
///     ColorSpace::Grayscale,
/// )?;
///
/// let out = BrickFilter::new(2, 1).apply(&img)?;
/// assert_eq!(out.pixels(), &[15, 15, 215, 215, 15, 15, 215, 215]);
///
/// let plan = BrickFilter::new(2, 1).plan(&img)?;
/// assert_eq!(plan.counts(), vec![([15, 15, 15], 1), ([215, 215, 215], 1)]);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrickFilter {
    columns: u32,
    rows: u32,
    max_colors: Option<u32>,
    background: BrickColor,
}

impl BrickFilter {
    /// Create a filter with a grid of `columns × rows` bricks.
    ///
    /// Parameters are validated when the filter is applied: both must be
    /// non-zero and must not exceed the image's width and height respectively.
    #[must_use]
    pub const fn new(columns: u32, rows: u32) -> Self {
        Self {
            columns,
            rows,
            max_colors: None,
            background: DEFAULT_BACKGROUND,
        }
    }

    /// Create a filter with a square grid of `n × n` bricks.
    #[must_use]
    pub const fn square(n: u32) -> Self {
        Self::new(n, n)
    }

    /// Limit the mosaic to at most `max_colors` distinct colors.
    ///
    /// Brick colors are reduced with median-cut. `0` is rejected when the
    /// filter is applied.
    #[must_use]
    pub const fn with_max_colors(mut self, max_colors: u32) -> Self {
        self.max_colors = Some(max_colors);
        self
    }

    /// Set the background color transparent pixels are flattened onto.
    ///
    /// Only affects `Rgba` images; `Grayscale` and `Rgb` have no transparency.
    #[must_use]
    pub const fn with_background(mut self, background: BrickColor) -> Self {
        self.background = background;
        self
    }

    /// Number of bricks across.
    #[must_use]
    pub const fn columns(&self) -> u32 {
        self.columns
    }

    /// Number of bricks down.
    #[must_use]
    pub const fn rows(&self) -> u32 {
        self.rows
    }

    /// Color limit, if any.
    #[must_use]
    pub const fn max_colors(&self) -> Option<u32> {
        self.max_colors
    }

    /// Background color used to flatten transparency.
    #[must_use]
    pub const fn background(&self) -> BrickColor {
        self.background
    }

    /// Compute the brick layout for `image` without rendering it.
    ///
    /// An empty image yields an empty plan (`0 × 0`, no cells).
    ///
    /// # Errors
    ///
    /// Returns [`SilvestreError::InvalidParameter`] if `columns` or `rows` is
    /// zero, if `max_colors` is zero, or if the grid is larger than the image.
    pub fn plan(&self, image: &SilvestreImage) -> Result<BrickPlan> {
        self.validate_params()?;

        let (width, height) = (image.width(), image.height());
        if width == 0 || height == 0 {
            return Ok(BrickPlan {
                columns: 0,
                rows: 0,
                cells: Vec::new(),
            });
        }

        if self.columns > width || self.rows > height {
            return Err(SilvestreError::InvalidParameter(format!(
                "brick grid {}x{} exceeds image size {width}x{height}",
                self.columns, self.rows
            )));
        }

        let mut cells = average_cells(image, self.columns, self.rows, self.background);

        if let Some(max_colors) = self.max_colors {
            reduce_colors(&mut cells, max_colors as usize);
        }

        Ok(BrickPlan {
            columns: self.columns,
            rows: self.rows,
            cells,
        })
    }

    fn validate_params(&self) -> Result<()> {
        if self.columns == 0 || self.rows == 0 {
            return Err(SilvestreError::InvalidParameter(
                "brick columns and rows must be non-zero".to_string(),
            ));
        }
        if self.max_colors == Some(0) {
            return Err(SilvestreError::InvalidParameter(
                "brick max_colors must be non-zero".to_string(),
            ));
        }
        Ok(())
    }
}

/// The brick layout computed by [`BrickFilter::plan`].
///
/// `cells` holds one color per brick in row-major order (left to right, top to
/// bottom), so `cells[row * columns + column]` is the brick at that position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrickPlan {
    columns: u32,
    rows: u32,
    cells: Vec<BrickColor>,
}

impl BrickPlan {
    /// Number of bricks across.
    #[must_use]
    pub const fn columns(&self) -> u32 {
        self.columns
    }

    /// Number of bricks down.
    #[must_use]
    pub const fn rows(&self) -> u32 {
        self.rows
    }

    /// Brick colors in row-major order.
    #[must_use]
    pub fn cells(&self) -> &[BrickColor] {
        &self.cells
    }

    /// Number of bricks needed of each color, most-used first.
    ///
    /// Colors with equal counts are ordered by their RGB value so the result is
    /// deterministic.
    #[must_use]
    pub fn counts(&self) -> Vec<(BrickColor, u32)> {
        let mut tally: BTreeMap<BrickColor, u32> = BTreeMap::new();
        for &color in &self.cells {
            *tally.entry(color).or_insert(0) += 1;
        }
        let mut counts: Vec<_> = tally.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        counts
    }

    /// Paint the plan onto an image of the given size and color space.
    fn render(&self, width: u32, height: u32, cs: ColorSpace) -> Result<SilvestreImage> {
        let channels = cs.channels();
        let (w, h) = (width as usize, height as usize);
        let mut dst = vec![0u8; w * h * channels];

        for row in 0..self.rows {
            let (y0, y1) = (
                edge(row, self.rows, height),
                edge(row + 1, self.rows, height),
            );
            for col in 0..self.columns {
                let (x0, x1) = (
                    edge(col, self.columns, width),
                    edge(col + 1, self.columns, width),
                );
                let [r, g, b] = self.cells[(row * self.columns + col) as usize];
                for y in y0..y1 {
                    for x in x0..x1 {
                        let offset = (y * w + x) * channels;
                        match cs {
                            ColorSpace::Grayscale => dst[offset] = r,
                            ColorSpace::Rgb => dst[offset..offset + 3].copy_from_slice(&[r, g, b]),
                            ColorSpace::Rgba => {
                                dst[offset..offset + 4].copy_from_slice(&[r, g, b, 255]);
                            }
                        }
                    }
                }
            }
        }

        SilvestreImage::new(dst, width, height, cs)
    }
}

/// Apply a `columns × rows` brick mosaic to `image` with default options.
///
/// Extracted as a free function so it can be called from tests and benchmarks
/// without constructing the filter struct.
///
/// # Errors
///
/// See [`BrickFilter::plan`].
pub fn brick(image: &SilvestreImage, columns: u32, rows: u32) -> Result<SilvestreImage> {
    BrickFilter::new(columns, rows).apply(image)
}

impl Filter for BrickFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let plan = self.plan(image)?;
        plan.render(image.width(), image.height(), image.color_space())
    }
}

/// Pixel offset of the `i`-th grid line when splitting `size` into `count`.
fn edge(i: u32, count: u32, size: u32) -> usize {
    (u64::from(i) * u64::from(size) / u64::from(count)) as usize
}

/// Composite `value` with coverage `alpha` over `bg`.
fn flatten(value: u8, alpha: u8, bg: u8) -> u32 {
    let a = u32::from(alpha);
    (u32::from(value) * a + u32::from(bg) * (255 - a) + 127) / 255
}

/// Average the flattened pixels of every brick, row-major.
fn average_cells(
    image: &SilvestreImage,
    columns: u32,
    rows: u32,
    background: BrickColor,
) -> Vec<BrickColor> {
    let cs = image.color_space();
    let channels = cs.channels();
    let src = image.pixels();
    let (width, height) = (image.width(), image.height());
    let w = width as usize;
    let mut cells = Vec::with_capacity(columns as usize * rows as usize);

    for row in 0..rows {
        let (y0, y1) = (edge(row, rows, height), edge(row + 1, rows, height));
        for col in 0..columns {
            let (x0, x1) = (edge(col, columns, width), edge(col + 1, columns, width));
            let mut sum = [0u64; 3];

            for y in y0..y1 {
                for x in x0..x1 {
                    let offset = (y * w + x) * channels;
                    let px: [u32; 3] = match cs {
                        ColorSpace::Grayscale => {
                            let v = u32::from(src[offset]);
                            [v, v, v]
                        }
                        ColorSpace::Rgb => [
                            u32::from(src[offset]),
                            u32::from(src[offset + 1]),
                            u32::from(src[offset + 2]),
                        ],
                        ColorSpace::Rgba => {
                            let a = src[offset + 3];
                            [
                                flatten(src[offset], a, background[0]),
                                flatten(src[offset + 1], a, background[1]),
                                flatten(src[offset + 2], a, background[2]),
                            ]
                        }
                    };
                    for (s, v) in sum.iter_mut().zip(px) {
                        *s += u64::from(v);
                    }
                }
            }

            let n = ((x1 - x0) * (y1 - y0)) as u64;
            cells.push(sum.map(|s| ((s + n / 2) / n) as u8));
        }
    }

    cells
}

/// Reduce `cells` to at most `max_colors` distinct colors using median-cut,
/// snapping every cell to the nearest palette color.
fn reduce_colors(cells: &mut [BrickColor], max_colors: usize) {
    let mut distinct: Vec<BrickColor> = cells.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    if distinct.len() <= max_colors {
        return;
    }

    let palette = median_cut(cells.to_vec(), max_colors);
    for cell in cells.iter_mut() {
        *cell = nearest(&palette, *cell);
    }
}

/// Split `colors` into at most `k` boxes and return the rounded mean of each.
fn median_cut(colors: Vec<BrickColor>, k: usize) -> Vec<BrickColor> {
    let mut boxes = vec![colors];

    while boxes.len() < k {
        // Split the box with the widest channel range; ties go to the first box.
        let Some((idx, channel, _)) = boxes
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let (ch, range) = widest_channel(b);
                (i, ch, range)
            })
            .filter(|&(_, _, range)| range > 0)
            .fold(None, |best: Option<(usize, usize, u8)>, cur| match best {
                Some(b) if b.2 >= cur.2 => Some(b),
                _ => Some(cur),
            })
        else {
            break;
        };

        let mut b = boxes.swap_remove(idx);
        b.sort_unstable_by(|p, q| p[channel].cmp(&q[channel]).then(p.cmp(q)));

        // Split near the median, but never between two equal channel values so
        // both halves are non-empty and disjoint along the split channel.
        let mut mid = b.len() / 2;
        while mid < b.len() && b[mid - 1][channel] == b[mid][channel] {
            mid += 1;
        }
        if mid == b.len() {
            mid = b.len() / 2;
            while b[mid - 1][channel] == b[mid][channel] {
                mid -= 1;
            }
        }

        let upper = b.split_off(mid);
        boxes.insert(idx, b);
        boxes.insert(idx + 1, upper);
    }

    boxes.iter().map(|b| mean(b)).collect()
}

/// The channel with the largest value range in `colors`, and that range.
fn widest_channel(colors: &[BrickColor]) -> (usize, u8) {
    (0..3)
        .map(|ch| {
            let (lo, hi) = colors.iter().fold((u8::MAX, u8::MIN), |(lo, hi), c| {
                (lo.min(c[ch]), hi.max(c[ch]))
            });
            (ch, hi.saturating_sub(lo))
        })
        .fold((0, 0), |best, cur| if cur.1 > best.1 { cur } else { best })
}

fn mean(colors: &[BrickColor]) -> BrickColor {
    let n = colors.len() as u64;
    let mut sum = [0u64; 3];
    for c in colors {
        for (s, v) in sum.iter_mut().zip(c) {
            *s += u64::from(*v);
        }
    }
    sum.map(|s| ((s + n / 2) / n) as u8)
}

/// Nearest palette color by squared Euclidean RGB distance; ties go to the
/// earliest palette entry.
fn nearest(palette: &[BrickColor], color: BrickColor) -> BrickColor {
    let dist = |p: &BrickColor| -> u32 {
        p.iter()
            .zip(color)
            .map(|(&a, b)| {
                let d = i32::from(a) - i32::from(b);
                (d * d) as u32
            })
            .sum()
    };
    *palette
        .iter()
        .min_by_key(|p| dist(p))
        .expect("palette is non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img(pixels: Vec<u8>, w: u32, h: u32, cs: ColorSpace) -> SilvestreImage {
        SilvestreImage::new(pixels, w, h, cs).unwrap()
    }

    // --- Known pixel values ---

    #[test]
    fn averages_each_brick_rgb() {
        // 2×1 RGB, one brick: average of (10,20,30) and (20,40,61).
        let image = img(vec![10, 20, 30, 20, 40, 61], 2, 1, ColorSpace::Rgb);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[15, 30, 46, 15, 30, 46]);
    }

    #[test]
    fn one_by_one_grid_is_whole_image_average() {
        let image = img(vec![0, 100, 200, 255], 2, 2, ColorSpace::Grayscale);
        let out = BrickFilter::square(1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[139; 4]);
    }

    #[test]
    fn grid_equal_to_image_size_is_identity() {
        let pixels = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let image = img(pixels.clone(), 2, 2, ColorSpace::Rgb);
        let out = BrickFilter::new(2, 2).apply(&image).unwrap();
        assert_eq!(out.pixels(), pixels.as_slice());
    }

    // --- Grid splitting ---

    #[test]
    fn uneven_split_covers_image_exactly() {
        // 5 px into 2 columns -> edges at 0, 2, 5 (widths 2 and 3).
        let image = img(vec![0, 0, 90, 90, 90], 5, 1, ColorSpace::Grayscale);
        let out = BrickFilter::new(2, 1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[0, 0, 90, 90, 90]);
    }

    #[test]
    fn multi_row_multi_column_layout() {
        // 4×4 grayscale, quadrants of 10/20/30/40, 2×2 grid.
        #[rustfmt::skip]
        let pixels = vec![
            10, 10, 20, 20,
            10, 10, 20, 20,
            30, 30, 40, 40,
            30, 30, 40, 40,
        ];
        let image = img(pixels.clone(), 4, 4, ColorSpace::Grayscale);
        let plan = BrickFilter::square(2).plan(&image).unwrap();
        assert_eq!(
            plan.cells(),
            &[[10, 10, 10], [20, 20, 20], [30, 30, 30], [40, 40, 40]]
        );
        let out = BrickFilter::square(2).apply(&image).unwrap();
        assert_eq!(out.pixels(), pixels.as_slice());
    }

    #[test]
    fn rectangular_bricks() {
        // 4×2 image, 2 columns × 2 rows -> 2×1 px bricks.
        #[rustfmt::skip]
        let pixels = vec![
            0, 20, 100, 120,
            50, 70, 200, 220,
        ];
        let image = img(pixels, 4, 2, ColorSpace::Grayscale);
        let out = BrickFilter::new(2, 2).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[10, 10, 110, 110, 60, 60, 210, 210]);
    }

    // --- Clamping ---

    #[test]
    fn extreme_values_stay_in_range() {
        let image = img(vec![255, 255, 255, 255], 4, 1, ColorSpace::Grayscale);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255; 4]);
        let image = img(vec![0, 0, 0, 0], 4, 1, ColorSpace::Grayscale);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[0; 4]);
    }

    // --- Color spaces ---

    #[test]
    fn grayscale_stays_single_channel() {
        let image = img(vec![10, 30], 2, 1, ColorSpace::Grayscale);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        assert_eq!(out.pixels(), &[20, 20]);
        let plan = BrickFilter::new(1, 1).plan(&image).unwrap();
        assert_eq!(plan.cells(), &[[20, 20, 20]]);
    }

    #[test]
    fn rgb_output() {
        let image = img(vec![255, 0, 0, 0, 0, 255], 2, 1, ColorSpace::Rgb);
        let out = BrickFilter::new(2, 1).apply(&image).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Rgb);
        assert_eq!(out.pixels(), &[255, 0, 0, 0, 0, 255]);
    }

    #[test]
    fn rgba_output_is_opaque() {
        let image = img(
            vec![10, 20, 30, 255, 40, 50, 60, 255],
            2,
            1,
            ColorSpace::Rgba,
        );
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Rgba);
        assert_eq!(out.pixels(), &[25, 35, 45, 255, 25, 35, 45, 255]);
    }

    // --- Alpha flattening ---

    #[test]
    fn transparent_pixels_flatten_onto_white() {
        let image = img(vec![0, 0, 0, 0], 1, 1, ColorSpace::Rgba);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        assert_eq!(out.pixels(), &[255, 255, 255, 255]);
    }

    #[test]
    fn half_transparent_red_becomes_pink() {
        let image = img(vec![255, 0, 0, 128], 1, 1, ColorSpace::Rgba);
        let out = BrickFilter::new(1, 1).apply(&image).unwrap();
        // 0·128 + 255·127 over 255 -> 127.
        assert_eq!(out.pixels(), &[255, 127, 127, 255]);
    }

    #[test]
    fn custom_background() {
        let image = img(vec![9, 9, 9, 0], 1, 1, ColorSpace::Rgba);
        let filter = BrickFilter::new(1, 1).with_background([0, 128, 255]);
        let out = filter.apply(&image).unwrap();
        assert_eq!(out.pixels(), &[0, 128, 255, 255]);
    }

    // --- Color reduction ---

    #[test]
    fn max_colors_limits_palette() {
        // Four bricks: two dark, two bright -> 2 colors.
        let image = img(vec![0, 10, 240, 250], 4, 1, ColorSpace::Grayscale);
        let plan = BrickFilter::new(4, 1)
            .with_max_colors(2)
            .plan(&image)
            .unwrap();
        assert_eq!(plan.counts(), vec![([5, 5, 5], 2), ([245, 245, 245], 2)]);
        assert_eq!(
            plan.cells(),
            &[[5, 5, 5], [5, 5, 5], [245, 245, 245], [245, 245, 245]]
        );
    }

    #[test]
    fn max_colors_not_reached_keeps_exact_colors() {
        let image = img(vec![0, 10, 240], 3, 1, ColorSpace::Grayscale);
        let plan = BrickFilter::new(3, 1)
            .with_max_colors(8)
            .plan(&image)
            .unwrap();
        assert_eq!(plan.cells(), &[[0, 0, 0], [10, 10, 10], [240, 240, 240]]);
    }

    #[test]
    fn max_colors_one_gives_single_color() {
        let image = img(vec![255, 0, 0, 0, 0, 255], 2, 1, ColorSpace::Rgb);
        let plan = BrickFilter::new(2, 1)
            .with_max_colors(1)
            .plan(&image)
            .unwrap();
        assert_eq!(plan.counts(), vec![([128, 0, 128], 2)]);
    }

    #[test]
    fn reduction_never_exceeds_max_colors() {
        let pixels: Vec<u8> = (0..=255u8).flat_map(|v| [v, 255 - v, v / 2]).collect();
        let image = img(pixels, 16, 16, ColorSpace::Rgb);
        for k in 1..=10 {
            let plan = BrickFilter::square(16)
                .with_max_colors(k)
                .plan(&image)
                .unwrap();
            assert!(plan.counts().len() <= k as usize, "k = {k}");
            let total: u32 = plan.counts().iter().map(|c| c.1).sum();
            assert_eq!(total, 256);
        }
    }

    #[test]
    fn reduction_is_deterministic() {
        let pixels: Vec<u8> = (0..64u8).flat_map(|v| [v * 4, v * 3, 255 - v]).collect();
        let image = img(pixels, 8, 8, ColorSpace::Rgb);
        let filter = BrickFilter::square(8).with_max_colors(5);
        assert_eq!(filter.plan(&image).unwrap(), filter.plan(&image).unwrap());
    }

    // --- Plan & counts ---

    #[test]
    fn counts_sorted_by_frequency_then_color() {
        let image = img(vec![50, 50, 50, 10, 200, 200], 6, 1, ColorSpace::Grayscale);
        let plan = BrickFilter::new(6, 1).plan(&image).unwrap();
        assert_eq!(
            plan.counts(),
            vec![([50, 50, 50], 3), ([200, 200, 200], 2), ([10, 10, 10], 1)]
        );
    }

    #[test]
    fn plan_dimensions_and_cell_count() {
        let image = img(vec![0; 6 * 4 * 3], 6, 4, ColorSpace::Rgb);
        let plan = BrickFilter::new(3, 2).plan(&image).unwrap();
        assert_eq!((plan.columns(), plan.rows()), (3, 2));
        assert_eq!(plan.cells().len(), 6);
    }

    #[test]
    fn apply_matches_plan() {
        let pixels: Vec<u8> = (0..48u8).map(|v| v * 5).collect();
        let image = img(pixels, 4, 4, ColorSpace::Rgb);
        let filter = BrickFilter::square(2).with_max_colors(2);
        let plan = filter.plan(&image).unwrap();
        let out = filter.apply(&image).unwrap();
        // Top-left pixel belongs to the first brick, bottom-right to the last.
        assert_eq!(&out.pixels()[0..3], &plan.cells()[0]);
        assert_eq!(&out.pixels()[45..48], &plan.cells()[3]);
    }

    // --- Output dimensions ---

    #[test]
    fn output_dimensions_unchanged() {
        let image = img(vec![7; 5 * 3 * 4], 5, 3, ColorSpace::Rgba);
        let out = BrickFilter::new(2, 2).apply(&image).unwrap();
        assert_eq!((out.width(), out.height()), (5, 3));
        assert_eq!(out.color_space(), ColorSpace::Rgba);
    }

    // --- Empty image ---

    #[test]
    fn empty_image_returns_ok() {
        let image = img(vec![], 0, 0, ColorSpace::Rgb);
        let out = BrickFilter::new(4, 4).apply(&image).unwrap();
        assert!(out.pixels().is_empty());
        let plan = BrickFilter::new(4, 4).plan(&image).unwrap();
        assert!(plan.cells().is_empty());
        assert!(plan.counts().is_empty());
    }

    // --- Errors ---

    #[test]
    fn zero_columns_or_rows_is_error() {
        let image = img(vec![0; 4], 2, 2, ColorSpace::Grayscale);
        assert!(BrickFilter::new(0, 1).apply(&image).is_err());
        assert!(BrickFilter::new(1, 0).apply(&image).is_err());
    }

    #[test]
    fn zero_max_colors_is_error() {
        let image = img(vec![0; 4], 2, 2, ColorSpace::Grayscale);
        assert!(BrickFilter::new(1, 1)
            .with_max_colors(0)
            .apply(&image)
            .is_err());
    }

    #[test]
    fn grid_larger_than_image_is_error() {
        let image = img(vec![0; 6], 3, 2, ColorSpace::Grayscale);
        assert!(BrickFilter::new(4, 1).apply(&image).is_err());
        assert!(BrickFilter::new(1, 3).apply(&image).is_err());
        assert!(BrickFilter::new(3, 2).apply(&image).is_ok());
    }

    // --- Filter trait ---

    #[test]
    fn filter_delegates_to_free_function() {
        let image = img(vec![1, 2, 3, 4, 5, 6, 7, 8], 4, 2, ColorSpace::Grayscale);
        let via_filter = BrickFilter::new(2, 1).apply(&image).unwrap();
        let via_fn = brick(&image, 2, 1).unwrap();
        assert_eq!(via_filter.pixels(), via_fn.pixels());
    }

    #[test]
    fn works_as_trait_object() {
        let image = img(vec![0, 100], 2, 1, ColorSpace::Grayscale);
        let filter: Box<dyn Filter> = Box::new(BrickFilter::new(1, 1));
        assert_eq!(filter.apply(&image).unwrap().pixels(), &[50, 50]);
    }

    // --- Accessors ---

    #[test]
    fn accessors_return_constructor_values() {
        let f = BrickFilter::new(12, 8)
            .with_max_colors(6)
            .with_background([1, 2, 3]);
        assert_eq!(f.columns(), 12);
        assert_eq!(f.rows(), 8);
        assert_eq!(f.max_colors(), Some(6));
        assert_eq!(f.background(), [1, 2, 3]);

        let d = BrickFilter::square(5);
        assert_eq!((d.columns(), d.rows()), (5, 5));
        assert_eq!(d.max_colors(), None);
        assert_eq!(d.background(), DEFAULT_BACKGROUND);
    }
}

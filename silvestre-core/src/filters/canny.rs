//! Canny edge detection filter.
//!
//! A multi-stage algorithm that produces thin, well-localized edges:
//!
//! 1. Convert to grayscale
//! 2. Gaussian blur (noise reduction)
//! 3. Sobel gradient (magnitude + direction)
//! 4. Non-maximum suppression (thin edges to 1px)
//! 5. Double threshold + hysteresis edge tracking

use crate::effects::grayscale::to_grayscale;
use crate::filters::gaussian::gaussian_blur;
use crate::filters::sobel::sobel_gradients;
use crate::filters::Filter;
use crate::{ColorSpace, Result, SilvestreError, SilvestreImage};

use std::f32::consts::PI;

/// Canny edge detection filter.
///
/// Configurable parameters:
/// - **`low_threshold`** / **`high_threshold`**: gradient magnitude thresholds
///   for the double-threshold step (0.0–255.0).
/// - **`sigma`**: standard deviation for the Gaussian blur stage.
///
/// # Examples
///
/// ```
/// use silvestre_core::filters::{CannyFilter, Filter};
/// use silvestre_core::{ColorSpace, SilvestreImage};
///
/// let img = SilvestreImage::new(vec![100; 25], 5, 5, ColorSpace::Grayscale)?;
/// let canny = CannyFilter::new(50.0, 100.0, 1.4)?;
/// let edges = canny.apply(&img)?;
/// assert_eq!(edges.color_space(), ColorSpace::Grayscale);
/// # Ok::<_, silvestre_core::SilvestreError>(())
/// ```
#[derive(Debug, Clone)]
pub struct CannyFilter {
    low_threshold: f32,
    high_threshold: f32,
    sigma: f32,
}

impl CannyFilter {
    /// Create a new Canny filter.
    ///
    /// - `low_threshold` and `high_threshold` must be in `0.0..=255.0` with
    ///   `low_threshold <= high_threshold`.
    /// - `sigma` must be a positive finite number.
    pub fn new(low_threshold: f32, high_threshold: f32, sigma: f32) -> Result<Self> {
        if !low_threshold.is_finite()
            || !high_threshold.is_finite()
            || low_threshold < 0.0
            || high_threshold < 0.0
            || low_threshold > 255.0
            || high_threshold > 255.0
        {
            return Err(SilvestreError::InvalidParameter(
                "thresholds must be finite values in 0.0..=255.0".into(),
            ));
        }
        if low_threshold > high_threshold {
            return Err(SilvestreError::InvalidParameter(
                "low_threshold must be <= high_threshold".into(),
            ));
        }
        if !sigma.is_finite() || sigma <= 0.0 {
            return Err(SilvestreError::InvalidParameter(
                "sigma must be a positive finite number".into(),
            ));
        }
        Ok(Self {
            low_threshold,
            high_threshold,
            sigma,
        })
    }

    /// Low gradient-magnitude threshold.
    #[must_use]
    pub fn low_threshold(&self) -> f32 {
        self.low_threshold
    }

    /// High gradient-magnitude threshold.
    #[must_use]
    pub fn high_threshold(&self) -> f32 {
        self.high_threshold
    }

    /// Gaussian blur sigma.
    #[must_use]
    pub fn sigma(&self) -> f32 {
        self.sigma
    }
}

impl Filter for CannyFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage> {
        let width = image.width();
        let height = image.height();

        if width == 0 || height == 0 {
            return SilvestreImage::new(vec![], width, height, ColorSpace::Grayscale);
        }

        // 1. Convert to grayscale.
        let gray = to_grayscale(image)?;

        // 2. Gaussian blur.
        let blurred = gaussian_blur(&gray, self.sigma)?;

        // 3. Sobel gradients.
        let (gx, gy) = sobel_gradients(&blurred, crate::filters::convolution::BorderMode::Clamp);
        let w = width as usize;
        let h = height as usize;
        let len = w * h;

        // Compute magnitude and direction.
        let mut magnitude = vec![0.0_f32; len];
        let mut direction = vec![0.0_f32; len];
        for i in 0..len {
            magnitude[i] = (gx[i] * gx[i] + gy[i] * gy[i]).sqrt();
            direction[i] = gy[i].atan2(gx[i]);
        }

        // 4. Non-maximum suppression.
        let suppressed = non_maximum_suppression(&magnitude, &direction, w, h);

        // 5. Double threshold + hysteresis.
        let edges = hysteresis(&suppressed, w, h, self.low_threshold, self.high_threshold);

        SilvestreImage::new(edges, width, height, ColorSpace::Grayscale)
    }
}

/// Quantize gradient direction to one of four edge orientations (0°, 45°,
/// 90°, 135°) and suppress non-maximum pixels along the gradient direction.
fn non_maximum_suppression(
    magnitude: &[f32],
    direction: &[f32],
    width: usize,
    height: usize,
) -> Vec<f32> {
    let mut result = vec![0.0_f32; magnitude.len()];

    for y in 1..height.saturating_sub(1) {
        for x in 1..width.saturating_sub(1) {
            let idx = y * width + x;
            let angle = normalize_angle(direction[idx]);
            let mag = magnitude[idx];

            // Determine the two neighbors along the gradient direction.
            let (n1, n2) = gradient_neighbors(angle, x, y, width);

            // Keep the pixel only if it is a local maximum.
            if mag >= magnitude[n1] && mag >= magnitude[n2] {
                result[idx] = mag;
            }
        }
    }

    result
}

/// Normalize an angle in radians to the range `[0, PI)` and quantize to the
/// nearest of 0°, 45°, 90°, 135°.
fn normalize_angle(angle: f32) -> u8 {
    // Map (-PI, PI] to [0, PI) by reflecting negative angles.
    let a = if angle < 0.0 { angle + PI } else { angle };
    // Quantize: 0°=[0,22.5)∪[157.5,180), 45°=[22.5,67.5), 90°=[67.5,112.5), 135°=[112.5,157.5)
    let deg = a * 180.0 / PI;
    if deg < 22.5 || deg >= 157.5 {
        0 // horizontal gradient -> compare left/right neighbors
    } else if deg < 67.5 {
        45
    } else if deg < 112.5 {
        90
    } else {
        135
    }
}

/// Return the indices of the two neighbor pixels along the gradient direction.
fn gradient_neighbors(angle: u8, x: usize, y: usize, width: usize) -> (usize, usize) {
    match angle {
        0 => {
            // Gradient is horizontal -> compare left and right.
            (y * width + (x - 1), y * width + (x + 1))
        }
        45 => {
            // Gradient is diagonal (NE-SW) -> compare top-right and bottom-left.
            ((y - 1) * width + (x + 1), (y + 1) * width + (x - 1))
        }
        90 => {
            // Gradient is vertical -> compare top and bottom.
            ((y - 1) * width + x, (y + 1) * width + x)
        }
        _ => {
            // 135°: Gradient is diagonal (NW-SE) -> compare top-left and bottom-right.
            ((y - 1) * width + (x - 1), (y + 1) * width + (x + 1))
        }
    }
}

/// Double threshold followed by hysteresis edge tracking.
///
/// Pixels above `high` are definite edges (255). Pixels below `low` are
/// suppressed (0). Pixels between `low` and `high` are kept only if they
/// are connected (8-connected) to a definite edge.
fn hysteresis(
    suppressed: &[f32],
    width: usize,
    height: usize,
    low: f32,
    high: f32,
) -> Vec<u8> {
    let len = width * height;
    // 0 = suppressed, 1 = weak, 2 = strong
    let mut marker = vec![0u8; len];

    // Classify pixels.
    for i in 0..len {
        if suppressed[i] >= high {
            marker[i] = 2;
        } else if suppressed[i] >= low {
            marker[i] = 1;
        }
    }

    // Hysteresis: flood-fill from strong edges to connected weak edges.
    // Use a stack-based approach to avoid recursion limits.
    let mut stack: Vec<usize> = Vec::new();
    for i in 0..len {
        if marker[i] == 2 {
            stack.push(i);
        }
    }

    while let Some(idx) = stack.pop() {
        let x = idx % width;
        let y = idx / width;
        // Check 8-connected neighbors.
        for dy in -1_i64..=1 {
            for dx in -1_i64..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i64 + dx;
                let ny = y as i64 + dy;
                if nx >= 0 && nx < width as i64 && ny >= 0 && ny < height as i64 {
                    let nidx = ny as usize * width + nx as usize;
                    if marker[nidx] == 1 {
                        marker[nidx] = 2;
                        stack.push(nidx);
                    }
                }
            }
        }
    }

    // Output: strong (promoted or original) = 255, everything else = 0.
    marker.iter().map(|&m| if m == 2 { 255 } else { 0 }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gray_image(width: u32, height: u32, pixels: Vec<u8>) -> SilvestreImage {
        SilvestreImage::new(pixels, width, height, ColorSpace::Grayscale).unwrap()
    }

    #[test]
    fn rejects_low_greater_than_high() {
        let err = CannyFilter::new(100.0, 50.0, 1.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_negative_thresholds() {
        let err = CannyFilter::new(-1.0, 50.0, 1.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_threshold_above_255() {
        let err = CannyFilter::new(0.0, 256.0, 1.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_nan_threshold() {
        let err = CannyFilter::new(f32::NAN, 100.0, 1.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_zero_sigma() {
        let err = CannyFilter::new(50.0, 100.0, 0.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn rejects_negative_sigma() {
        let err = CannyFilter::new(50.0, 100.0, -1.0).unwrap_err();
        assert!(matches!(err, SilvestreError::InvalidParameter(_)));
    }

    #[test]
    fn accessors() {
        let f = CannyFilter::new(30.0, 90.0, 1.4).unwrap();
        assert!((f.low_threshold() - 30.0).abs() < f32::EPSILON);
        assert!((f.high_threshold() - 90.0).abs() < f32::EPSILON);
        assert!((f.sigma() - 1.4).abs() < f32::EPSILON);
    }

    #[test]
    fn equal_thresholds_accepted() {
        assert!(CannyFilter::new(50.0, 50.0, 1.0).is_ok());
    }

    #[test]
    fn uniform_image_no_edges() {
        let img = gray_image(10, 10, vec![128; 100]);
        let canny = CannyFilter::new(50.0, 100.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();
        assert!(
            out.pixels().iter().all(|&v| v == 0),
            "uniform image should produce no edges"
        );
    }

    #[test]
    fn white_rectangle_on_black_produces_edges() {
        // 20x20 black image with a 10x10 white rectangle in the center.
        let mut pixels = vec![0u8; 20 * 20];
        for y in 5..15 {
            for x in 5..15 {
                pixels[y * 20 + x] = 255;
            }
        }
        let img = gray_image(20, 20, pixels);
        let canny = CannyFilter::new(30.0, 100.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();

        // There should be some edge pixels (255) in the output.
        let edge_count = out.pixels().iter().filter(|&&v| v == 255).count();
        assert!(
            edge_count > 0,
            "rectangle should produce visible edges, got 0"
        );

        // The output should be binary: only 0 or 255.
        assert!(
            out.pixels().iter().all(|&v| v == 0 || v == 255),
            "output must be binary"
        );

        // Interior of the rectangle (well inside the edges) should be 0.
        assert_eq!(
            out.pixels()[10 * 20 + 10],
            0,
            "interior pixel should not be an edge"
        );
        // Exterior (well outside) should also be 0.
        assert_eq!(
            out.pixels()[1 * 20 + 1],
            0,
            "exterior pixel should not be an edge"
        );
    }

    #[test]
    fn accepts_rgb_input() {
        // Canny should auto-convert to grayscale.
        let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 128, 128, 128];
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgb).unwrap();
        let canny = CannyFilter::new(20.0, 80.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
        assert_eq!(out.width(), 2);
        assert_eq!(out.height(), 2);
    }

    #[test]
    fn accepts_rgba_input() {
        let pixels = vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 128, 128, 128, 255];
        let img = SilvestreImage::new(pixels, 2, 2, ColorSpace::Rgba).unwrap();
        let canny = CannyFilter::new(20.0, 80.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();
        assert_eq!(out.color_space(), ColorSpace::Grayscale);
    }

    #[test]
    fn empty_image() {
        let img = gray_image(0, 0, vec![]);
        let canny = CannyFilter::new(50.0, 100.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();
        assert!(out.pixels().is_empty());
    }

    #[test]
    fn preserves_dimensions() {
        let img = gray_image(15, 10, vec![50; 150]);
        let canny = CannyFilter::new(50.0, 100.0, 1.4).unwrap();
        let out = canny.apply(&img).unwrap();
        assert_eq!(out.width(), 15);
        assert_eq!(out.height(), 10);
    }

    #[test]
    fn filter_trait_object() {
        let filter: Box<dyn Filter> = Box::new(CannyFilter::new(50.0, 100.0, 1.4).unwrap());
        let img = gray_image(5, 5, vec![50; 25]);
        let out = filter.apply(&img).unwrap();
        assert_eq!(out.width(), 5);
    }

    #[test]
    fn strong_edge_line() {
        // A single bright horizontal stripe in the middle of a dark image
        // should produce edge pixels above and below the stripe.
        let mut pixels = vec![0u8; 15 * 15];
        for x in 0..15 {
            pixels[7 * 15 + x] = 255;
        }
        let img = gray_image(15, 15, pixels);
        let canny = CannyFilter::new(20.0, 60.0, 1.0).unwrap();
        let out = canny.apply(&img).unwrap();
        let edge_count = out.pixels().iter().filter(|&&v| v == 255).count();
        assert!(edge_count > 0, "horizontal stripe should produce edges");
    }

    #[test]
    fn hysteresis_connects_weak_to_strong() {
        // Manually test hysteresis: a strong pixel adjacent to weak pixels
        // should promote the weak ones.
        let suppressed = vec![
            0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 40.0, 0.0, 0.0,
            0.0, 40.0, 120.0, 40.0, 0.0,
            0.0, 0.0, 40.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0, 0.0,
        ];
        let result = hysteresis(&suppressed, 5, 5, 30.0, 100.0);
        // Center (2,2) is strong (120 >= 100) -> 255.
        assert_eq!(result[12], 255);
        // Its neighbors (40 >= 30 = weak) connected to strong -> 255.
        assert_eq!(result[7], 255); // (2,1)
        assert_eq!(result[11], 255); // (1,2)
        assert_eq!(result[13], 255); // (3,2)
        assert_eq!(result[17], 255); // (2,3)
        // Far corners are 0.
        assert_eq!(result[0], 0);
        assert_eq!(result[24], 0);
    }

    #[test]
    fn weak_without_strong_neighbor_suppressed() {
        // All pixels are in the weak range but none are strong -> all suppressed.
        let suppressed = vec![
            0.0, 0.0, 0.0,
            0.0, 60.0, 0.0,
            0.0, 0.0, 0.0,
        ];
        let result = hysteresis(&suppressed, 3, 3, 30.0, 100.0);
        assert!(result.iter().all(|&v| v == 0), "isolated weak pixel should be suppressed");
    }

    #[test]
    fn normalize_angle_quantization() {
        // 0° direction
        assert_eq!(normalize_angle(0.0), 0);
        // ~45° direction
        assert_eq!(normalize_angle(PI / 4.0), 45);
        // ~90° direction
        assert_eq!(normalize_angle(PI / 2.0), 90);
        // ~135° direction
        assert_eq!(normalize_angle(3.0 * PI / 4.0), 135);
        // Negative angle maps same as positive reflection
        assert_eq!(normalize_angle(-PI / 4.0), 135);
    }
}

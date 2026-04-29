# RotateFilter Design Specification

**Date:** 2026-04-28
**Issue:** #19
**Status:** Design Review

## Overview

Implement image rotation for fixed angles (90°, 180°, 270°) and arbitrary angles with bilinear interpolation. The filter preserves color space and multi-channel support while providing optimized fast paths for common fixed rotations.

## Requirements

- Fixed rotations (90°, 180°, 270°) must be fast, lossless, and swap dimensions appropriately
- Arbitrary angle rotation uses bilinear interpolation
- Output canvas stays same size as input (arbitrary angles)
- Configurable background fill color for areas outside source bounds
- Support grayscale and RGB/RGBA color spaces
- Implement `Filter` trait
- Round-trip and 360° rotation correctness

## Module Structure & API

### File Location
`silvestre-core/src/transform/rotate.rs`

### Public Interface

```rust
pub struct RotateFilter {
    angle: f64,
    grayscale_background: u8,
    rgb_background: [u8; 3],
}

impl RotateFilter {
    /// Create a new rotation filter.
    ///
    /// `angle`: Rotation angle in degrees (0-360, automatically normalized)
    /// `grayscale_background`: Background value for grayscale images (0-255)
    /// `rgb_background`: Background RGB values for color images
    #[must_use]
    pub fn new(angle: f64, grayscale_background: u8, rgb_background: [u8; 3]) -> Self

    #[must_use]
    pub fn angle(&self) -> f64

    #[must_use]
    pub fn grayscale_background(&self) -> u8

    #[must_use]
    pub fn rgb_background(&self) -> [u8; 3]
}

impl Filter for RotateFilter {
    fn apply(&self, image: &SilvestreImage) -> Result<SilvestreImage>
}
```

### Module Exports
Update `silvestre-core/src/transform/mod.rs`:
```rust
pub mod rotate;
pub use rotate::RotateFilter;
```

## Implementation Approach

### Two-Path Architecture

The `apply` method uses path-based dispatch for efficiency:

#### Path 1: Fixed Angle Optimization (0°, 90°, 180°, 270°)

1. **Angle normalization**: Reduce to 0-360 range using modulo
2. **Fixed angle detection**: Check if angle is within floating-point tolerance of 0, 90, 180, or 270
3. **Implementation per angle:**
   - **0°**: Return image clone (identity)
   - **90°**: Rotate counter-clockwise, swap dimensions (W×H → H×W), rearrange pixels
   - **180°**: Equivalent to `MirrorFilter::Both` (horizontal + vertical flip)
   - **270°**: Rotate clockwise, swap dimensions, rearrange pixels

**Key property:** Fixed rotations are lossless and preserve exact pixel values.

#### Path 2: Arbitrary Angle with Bilinear Interpolation

For non-fixed angles:

1. **Compute rotation parameters:**
   - `cos_angle = cos(angle_in_radians)`
   - `sin_angle = sin(angle_in_radians)`
   - Image center: `cx = width / 2.0`, `cy = height / 2.0`

2. **For each destination pixel** `(dst_x, dst_y)`:
   - Translate to image center: `px = dst_x - cx`, `py = dst_y - cy`
   - Apply inverse rotation (negate angle):
     ```
     src_x = px * cos_angle + py * sin_angle + cx
     src_y = -px * sin_angle + py * cos_angle + cy
     ```
   - Check bounds: if `src_x, src_y` outside `[0, width) × [0, height)`, use background color
   - Otherwise, use bilinear interpolation to sample from source at `(src_x, src_y)`
   - Write interpolated value to destination

3. **Bilinear interpolation:**
   - Get floor coordinates: `x0 = floor(src_x)`, `y0 = floor(src_y)`
   - Fractional parts: `fx = src_x - x0`, `fy = src_y - y0`
   - Clamp integer coords to valid range
   - Blend four neighbors: `top_left, top_right, bottom_left, bottom_right`
   - Per-channel linear blend using `fx` and `fy`

### Background Color Selection at Runtime

Based on image color space:
- **Grayscale**: Use `grayscale_background` for the single channel
- **RGB**: Use `rgb_background[0..3]` for all pixels
- **RGBA**: Use `rgb_background[0..3]` for color, `255` for alpha (opaque)

### Edge Cases & Error Handling

| Case | Behavior |
|------|----------|
| Empty image (0×0) | Return early with empty image |
| Single pixel | Return clone (rotation has no effect) |
| Angle = NaN | Treat as error or undefined (needs validation) |
| Very large angle (e.g., 720°) | Normalize and proceed |

No allocation errors expected since output dimensions equal input dimensions.

## Testing Strategy

### Fixed Rotation Tests (90°, 180°, 270°)

- **Dimension swap:** 90° on 4×3 image produces 3×4
- **Pixel position correctness:** Asymmetric 3×3 image verifies exact pixel locations
- **Round-trip:** Four 90° rotations return original
- **270° equivalence:** Three 90° rotations equal one 270°
- **180° equivalence:** Same as `MirrorFilter::Both`
- **Edge cases:** Single pixel, single row, single column
- **Multi-channel:** Grayscale, RGB, RGBA all work correctly

### Arbitrary Rotation Tests (45°, 30°, etc.)

- **Background fill:** Corners are filled with background color
- **Bounds checking:** Out-of-bounds samples use background, not errors
- **Interpolation quality:** Small rotations preserve most pixels, boundaries smooth
- **Grayscale vs RGB:** Background color correctly adapted per color space
- **360° round-trip:** Returns identical or near-identical image (allow small interpolation variance)
- **Out-of-center rotation:** Verify rotation happens around image center

### General Tests

- **Empty image:** 0×0 input returns 0×0 output
- **Metadata preservation:** Width, height, color space unchanged (except 90°/270° swap width/height)
- **Filter trait object:** Can be used as `Box<dyn Filter>`
- **Consistency:** Same input + angle produces same output on multiple calls

## Acceptance Criteria

- [ ] 90° rotation swaps width and height correctly
- [ ] 360° rotation returns original image
- [ ] Four 90° rotations return original image (round-trip)
- [ ] Arbitrary angle rotation fills empty areas with specified color
- [ ] Unit tests with small known images verify pixel positions after rotation
- [ ] All tests pass: `cargo test -p silvestre-core`
- [ ] Workspace builds: `cargo build --workspace`

## Future Considerations

- **Optimization:** SIMD for bilinear interpolation (future)
- **Output canvas:** Option to expand canvas for arbitrary angles (future enhancement)
- **Rotation direction:** Parameterize CW vs CCW (currently fixed)
- **Interpolation methods:** Add nearest-neighbor option (future)

---

**Next Phase:** Invoke `writing-plans` skill to generate implementation plan with detailed step-by-step tasks.

//! Hardware SIMD acceleration subsystem.
//!
//! Provides optimized vector kernels for color transformations, saturated
//! arithmetic, and pixel format conversions across x86_64, AArch64 (NEON),
//! and WebAssembly SIMD128, with 100% equivalent scalar fallbacks.

pub mod aarch64;
pub mod scalar;
pub mod wasm;
pub mod x86_64;

/// Inverts color channels in `src` and writes the result into `dst`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn invert(src: &[u8], dst: &mut [u8], channels: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::invert(src, dst, channels);
    }
    #[cfg(target_arch = "x86_64")]
    {
        x86_64::invert(src, dst, channels);
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        wasm::invert(src, dst, channels);
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        scalar::invert(src, dst, channels);
    }
}

/// Saturated brightness addition: `dst = clamp(src + delta, 0, 255)`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn brightness_add(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::brightness_add(src, dst, delta, channels);
    }
    #[cfg(target_arch = "x86_64")]
    {
        x86_64::brightness_add(src, dst, delta, channels);
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        wasm::brightness_add(src, dst, delta, channels);
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        scalar::brightness_add(src, dst, delta, channels);
    }
}

/// Saturated brightness subtraction: `dst = clamp(src - delta, 0, 255)`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn brightness_sub(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::brightness_sub(src, dst, delta, channels);
    }
    #[cfg(target_arch = "x86_64")]
    {
        x86_64::brightness_sub(src, dst, delta, channels);
    }
    #[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
    {
        wasm::brightness_sub(src, dst, delta, channels);
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "x86_64",
        all(target_arch = "wasm32", target_feature = "simd128")
    )))]
    {
        scalar::brightness_sub(src, dst, delta, channels);
    }
}

/// Converts interleaved RGB pixels to single-channel Grayscale using ITU-R BT.601.
#[inline]
pub fn grayscale_rgb(src: &[u8], dst: &mut [u8]) {
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::grayscale_rgb(src, dst);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        scalar::grayscale_rgb(src, dst);
    }
}

/// Converts interleaved RGBA pixels to single-channel Grayscale (alpha discarded).
#[inline]
pub fn grayscale_rgba(src: &[u8], dst: &mut [u8]) {
    #[cfg(target_arch = "aarch64")]
    {
        aarch64::grayscale_rgba(src, dst);
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        scalar::grayscale_rgba(src, dst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invert_equivalence_rgb() {
        let sizes = [
            0, 1, 3, 7, 15, 16, 17, 31, 32, 33, 48, 64, 100, 256, 512, 1024,
        ];
        for size in sizes {
            let src: Vec<u8> = (0..size).map(|i| (i * 37 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; size];
            let mut dst_simd = vec![0u8; size];

            scalar::invert(&src, &mut dst_scalar, 3);
            invert(&src, &mut dst_simd, 3);

            assert_eq!(dst_scalar, dst_simd, "failed at size {}", size);
        }
    }

    #[test]
    fn test_invert_equivalence_rgba() {
        let pixel_counts = [0, 1, 2, 3, 4, 5, 7, 8, 9, 16, 32, 64, 100, 256];
        for count in pixel_counts {
            let size = count * 4;
            let src: Vec<u8> = (0..size).map(|i| (i * 73 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; size];
            let mut dst_simd = vec![0u8; size];

            scalar::invert(&src, &mut dst_scalar, 4);
            invert(&src, &mut dst_simd, 4);

            assert_eq!(dst_scalar, dst_simd, "failed at RGBA pixel count {}", count);
        }
    }

    #[test]
    fn test_brightness_add_equivalence() {
        let sizes = [0, 1, 3, 4, 15, 16, 32, 64, 128, 256, 512];
        for size in sizes {
            let src: Vec<u8> = (0..size).map(|i| (i * 19 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; size];
            let mut dst_simd = vec![0u8; size];

            scalar::brightness_add(&src, &mut dst_scalar, 50, 3);
            brightness_add(&src, &mut dst_simd, 50, 3);

            assert_eq!(dst_scalar, dst_simd, "failed at size {}", size);
        }
    }

    #[test]
    fn test_brightness_sub_equivalence() {
        let sizes = [0, 1, 3, 4, 15, 16, 32, 64, 128, 256, 512];
        for size in sizes {
            let src: Vec<u8> = (0..size).map(|i| (i * 19 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; size];
            let mut dst_simd = vec![0u8; size];

            scalar::brightness_sub(&src, &mut dst_scalar, 50, 3);
            brightness_sub(&src, &mut dst_simd, 50, 3);

            assert_eq!(dst_scalar, dst_simd, "failed at size {}", size);
        }
    }

    #[test]
    fn test_grayscale_rgb_equivalence() {
        let pixel_counts = [0, 1, 2, 3, 4, 8, 15, 16, 17, 32, 64, 128, 256];
        for count in pixel_counts {
            let src: Vec<u8> = (0..count * 3).map(|i| (i * 13 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; count];
            let mut dst_simd = vec![0u8; count];

            scalar::grayscale_rgb(&src, &mut dst_scalar);
            grayscale_rgb(&src, &mut dst_simd);

            // Allow at most 1 unit difference due to floating point vs fixed-point rounding
            for (i, (&s, &v)) in dst_scalar.iter().zip(dst_simd.iter()).enumerate() {
                assert!(
                    (s as i16 - v as i16).abs() <= 1,
                    "grayscale RGB mismatch at index {}: scalar={}, simd={}",
                    i,
                    s,
                    v
                );
            }
        }
    }

    #[test]
    fn test_grayscale_rgba_equivalence() {
        let pixel_counts = [0, 1, 2, 3, 4, 8, 15, 16, 17, 32, 64, 128, 256];
        for count in pixel_counts {
            let src: Vec<u8> = (0..count * 4).map(|i| (i * 17 % 256) as u8).collect();
            let mut dst_scalar = vec![0u8; count];
            let mut dst_simd = vec![0u8; count];

            scalar::grayscale_rgba(&src, &mut dst_scalar);
            grayscale_rgba(&src, &mut dst_simd);

            for (i, (&s, &v)) in dst_scalar.iter().zip(dst_simd.iter()).enumerate() {
                assert!(
                    (s as i16 - v as i16).abs() <= 1,
                    "grayscale RGBA mismatch at index {}: scalar={}, simd={}",
                    i,
                    s,
                    v
                );
            }
        }
    }
}

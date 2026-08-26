//! x86_64 AVX2 & SSE2 SIMD implementations with runtime feature detection.

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
const RGBA_MASK_128: [i8; 16] = [
    -1, -1, -1, 0, // Pixel 0
    -1, -1, -1, 0, // Pixel 1
    -1, -1, -1, 0, // Pixel 2
    -1, -1, -1, 0, // Pixel 3
];

#[cfg(target_arch = "x86_64")]
const RGBA_MASK_256: [i8; 32] = [
    -1, -1, -1, 0, -1, -1, -1, 0, -1, -1, -1, 0, -1, -1, -1, 0, // Pixels 0..3
    -1, -1, -1, 0, -1, -1, -1, 0, -1, -1, -1, 0, -1, -1, -1, 0, // Pixels 4..7
];

/// Invert color channels using x86_64 AVX2 / SSE2 intrinsics.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn invert(src: &[u8], dst: &mut [u8], channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    if is_x86_feature_detected!("avx2") {
        unsafe {
            if channels == 4 {
                let mask = _mm256_loadu_si256(RGBA_MASK_256.as_ptr() as *const __m256i);
                let ones = _mm256_set1_epi8(-1);
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let inverted = _mm256_xor_si256(s, ones);
                    let result = _mm256_blendv_epi8(s, inverted, mask);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, result);
                    i += 32;
                }
            } else {
                let ones = _mm256_set1_epi8(-1);
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let inverted = _mm256_xor_si256(s, ones);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, inverted);
                    i += 32;
                }
            }
        }
    }

    // SSE2 (128-bit) loop
    unsafe {
        if channels == 4 {
            let mask = _mm_loadu_si128(RGBA_MASK_128.as_ptr() as *const __m128i);
            let ones = _mm_set1_epi8(-1);
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let inverted = _mm_xor_si128(s, ones);
                // In SSE2, blend via bitwise operations: (inverted & mask) | (s & ~mask)
                let blended =
                    _mm_or_si128(_mm_and_si128(inverted, mask), _mm_andnot_si128(mask, s));
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, blended);
                i += 16;
            }
        } else {
            let ones = _mm_set1_epi8(-1);
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let inverted = _mm_xor_si128(s, ones);
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, inverted);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::invert(&src[i..len], &mut dst[i..len], channels);
    }
}

/// Saturated brightness addition using x86_64 AVX2 / SSE2 intrinsics.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn brightness_add(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    if is_x86_feature_detected!("avx2") {
        unsafe {
            let delta_vec = _mm256_set1_epi8(delta as i8);
            if channels == 4 {
                let mask = _mm256_loadu_si256(RGBA_MASK_256.as_ptr() as *const __m256i);
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let added = _mm256_adds_epu8(s, delta_vec);
                    let result = _mm256_blendv_epi8(s, added, mask);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, result);
                    i += 32;
                }
            } else {
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let added = _mm256_adds_epu8(s, delta_vec);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, added);
                    i += 32;
                }
            }
        }
    }

    unsafe {
        let delta_vec = _mm_set1_epi8(delta as i8);
        if channels == 4 {
            let mask = _mm_loadu_si128(RGBA_MASK_128.as_ptr() as *const __m128i);
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let added = _mm_adds_epu8(s, delta_vec);
                let blended = _mm_or_si128(_mm_and_si128(added, mask), _mm_andnot_si128(mask, s));
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, blended);
                i += 16;
            }
        } else {
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let added = _mm_adds_epu8(s, delta_vec);
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, added);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_add(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

/// Saturated brightness subtraction using x86_64 AVX2 / SSE2 intrinsics.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn brightness_sub(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    if is_x86_feature_detected!("avx2") {
        unsafe {
            let delta_vec = _mm256_set1_epi8(delta as i8);
            if channels == 4 {
                let mask = _mm256_loadu_si256(RGBA_MASK_256.as_ptr() as *const __m256i);
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let subbed = _mm256_subs_epu8(s, delta_vec);
                    let result = _mm256_blendv_epi8(s, subbed, mask);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, result);
                    i += 32;
                }
            } else {
                while i + 32 <= len {
                    let s = _mm256_loadu_si256(src.as_ptr().add(i) as *const __m256i);
                    let subbed = _mm256_subs_epu8(s, delta_vec);
                    _mm256_storeu_si256(dst.as_mut_ptr().add(i) as *mut __m256i, subbed);
                    i += 32;
                }
            }
        }
    }

    unsafe {
        let delta_vec = _mm_set1_epi8(delta as i8);
        if channels == 4 {
            let mask = _mm_loadu_si128(RGBA_MASK_128.as_ptr() as *const __m128i);
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let subbed = _mm_subs_epu8(s, delta_vec);
                let blended = _mm_or_si128(_mm_and_si128(subbed, mask), _mm_andnot_si128(mask, s));
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, blended);
                i += 16;
            }
        } else {
            while i + 16 <= len {
                let s = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
                let subbed = _mm_subs_epu8(s, delta_vec);
                _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, subbed);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_sub(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

/// Grayscale conversion for interleaved RGB pixels on x86_64.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn grayscale_rgb(src: &[u8], dst: &mut [u8]) {
    super::scalar::grayscale_rgb(src, dst);
}

/// Grayscale conversion for interleaved RGBA pixels on x86_64.
#[cfg(target_arch = "x86_64")]
#[inline]
pub fn grayscale_rgba(src: &[u8], dst: &mut [u8]) {
    super::scalar::grayscale_rgba(src, dst);
}

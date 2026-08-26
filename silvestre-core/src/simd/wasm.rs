//! WebAssembly SIMD128 implementation for wasm32 targets.

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
use core::arch::wasm32::*;

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
const RGBA_MASK: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 0
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 1
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 2
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 3
];

/// Invert color channels using WASM SIMD128 intrinsics.
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[inline]
pub fn invert(src: &[u8], dst: &mut [u8], channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        if channels == 4 {
            let mask = v128_load(RGBA_MASK.as_ptr() as *const v128);
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let inv = v128_not(s);
                let result = v128_bitselect(inv, s, mask);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, result);
                i += 16;
            }
        } else {
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let inv = v128_not(s);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, inv);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::invert(&src[i..len], &mut dst[i..len], channels);
    }
}

/// Saturated brightness addition using WASM SIMD128 intrinsics.
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[inline]
pub fn brightness_add(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        let delta_vec = u8x16_splat(delta);
        if channels == 4 {
            let mask = v128_load(RGBA_MASK.as_ptr() as *const v128);
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let added = u8x16_add_sat(s, delta_vec);
                let result = v128_bitselect(added, s, mask);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, result);
                i += 16;
            }
        } else {
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let added = u8x16_add_sat(s, delta_vec);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, added);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_add(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

/// Saturated brightness subtraction using WASM SIMD128 intrinsics.
#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
#[inline]
pub fn brightness_sub(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        let delta_vec = u8x16_splat(delta);
        if channels == 4 {
            let mask = v128_load(RGBA_MASK.as_ptr() as *const v128);
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let subbed = u8x16_sub_sat(s, delta_vec);
                let result = v128_bitselect(subbed, s, mask);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, result);
                i += 16;
            }
        } else {
            while i + 16 <= len {
                let s = v128_load(src.as_ptr().add(i) as *const v128);
                let subbed = u8x16_sub_sat(s, delta_vec);
                v128_store(dst.as_mut_ptr().add(i) as *mut v128, subbed);
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_sub(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

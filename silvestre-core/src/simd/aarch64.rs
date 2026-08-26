//! ARM NEON 128-bit SIMD implementation for AArch64.

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;

#[cfg(target_arch = "aarch64")]
const RGBA_MASK: [u8; 16] = [
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 0 (RGB inverted/adjusted, Alpha kept)
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 1
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 2
    0xFF, 0xFF, 0xFF, 0x00, // Pixel 3
];

/// Invert color channels using ARM NEON intrinsics with 4x unrolling.
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn invert(src: &[u8], dst: &mut [u8], channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        if channels == 4 {
            let mask = vld1q_u8(RGBA_MASK.as_ptr());
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                let r0 = vbslq_u8(mask, vmvnq_u8(s0), s0);
                let r1 = vbslq_u8(mask, vmvnq_u8(s1), s1);
                let r2 = vbslq_u8(mask, vmvnq_u8(s2), s2);
                let r3 = vbslq_u8(mask, vmvnq_u8(s3), s3);

                vst1q_u8(dst.as_mut_ptr().add(i), r0);
                vst1q_u8(dst.as_mut_ptr().add(i + 16), r1);
                vst1q_u8(dst.as_mut_ptr().add(i + 32), r2);
                vst1q_u8(dst.as_mut_ptr().add(i + 48), r3);

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                let result = vbslq_u8(mask, vmvnq_u8(s), s);
                vst1q_u8(dst.as_mut_ptr().add(i), result);
                i += 16;
            }
        } else {
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                vst1q_u8(dst.as_mut_ptr().add(i), vmvnq_u8(s0));
                vst1q_u8(dst.as_mut_ptr().add(i + 16), vmvnq_u8(s1));
                vst1q_u8(dst.as_mut_ptr().add(i + 32), vmvnq_u8(s2));
                vst1q_u8(dst.as_mut_ptr().add(i + 48), vmvnq_u8(s3));

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                vst1q_u8(dst.as_mut_ptr().add(i), vmvnq_u8(s));
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::invert(&src[i..len], &mut dst[i..len], channels);
    }
}

/// Saturated brightness addition using ARM NEON intrinsics with 4x unrolling.
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn brightness_add(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        let delta_vec = vdupq_n_u8(delta);
        if channels == 4 {
            let mask = vld1q_u8(RGBA_MASK.as_ptr());
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                let r0 = vbslq_u8(mask, vqaddq_u8(s0, delta_vec), s0);
                let r1 = vbslq_u8(mask, vqaddq_u8(s1, delta_vec), s1);
                let r2 = vbslq_u8(mask, vqaddq_u8(s2, delta_vec), s2);
                let r3 = vbslq_u8(mask, vqaddq_u8(s3, delta_vec), s3);

                vst1q_u8(dst.as_mut_ptr().add(i), r0);
                vst1q_u8(dst.as_mut_ptr().add(i + 16), r1);
                vst1q_u8(dst.as_mut_ptr().add(i + 32), r2);
                vst1q_u8(dst.as_mut_ptr().add(i + 48), r3);

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                let result = vbslq_u8(mask, vqaddq_u8(s, delta_vec), s);
                vst1q_u8(dst.as_mut_ptr().add(i), result);
                i += 16;
            }
        } else {
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                vst1q_u8(dst.as_mut_ptr().add(i), vqaddq_u8(s0, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 16), vqaddq_u8(s1, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 32), vqaddq_u8(s2, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 48), vqaddq_u8(s3, delta_vec));

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                vst1q_u8(dst.as_mut_ptr().add(i), vqaddq_u8(s, delta_vec));
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_add(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

/// Saturated brightness subtraction using ARM NEON intrinsics with 4x unrolling.
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn brightness_sub(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    let mut i = 0;

    unsafe {
        let delta_vec = vdupq_n_u8(delta);
        if channels == 4 {
            let mask = vld1q_u8(RGBA_MASK.as_ptr());
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                let r0 = vbslq_u8(mask, vqsubq_u8(s0, delta_vec), s0);
                let r1 = vbslq_u8(mask, vqsubq_u8(s1, delta_vec), s1);
                let r2 = vbslq_u8(mask, vqsubq_u8(s2, delta_vec), s2);
                let r3 = vbslq_u8(mask, vqsubq_u8(s3, delta_vec), s3);

                vst1q_u8(dst.as_mut_ptr().add(i), r0);
                vst1q_u8(dst.as_mut_ptr().add(i + 16), r1);
                vst1q_u8(dst.as_mut_ptr().add(i + 32), r2);
                vst1q_u8(dst.as_mut_ptr().add(i + 48), r3);

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                let result = vbslq_u8(mask, vqsubq_u8(s, delta_vec), s);
                vst1q_u8(dst.as_mut_ptr().add(i), result);
                i += 16;
            }
        } else {
            while i + 64 <= len {
                let s0 = vld1q_u8(src.as_ptr().add(i));
                let s1 = vld1q_u8(src.as_ptr().add(i + 16));
                let s2 = vld1q_u8(src.as_ptr().add(i + 32));
                let s3 = vld1q_u8(src.as_ptr().add(i + 48));

                vst1q_u8(dst.as_mut_ptr().add(i), vqsubq_u8(s0, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 16), vqsubq_u8(s1, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 32), vqsubq_u8(s2, delta_vec));
                vst1q_u8(dst.as_mut_ptr().add(i + 48), vqsubq_u8(s3, delta_vec));

                i += 64;
            }
            while i + 16 <= len {
                let s = vld1q_u8(src.as_ptr().add(i));
                vst1q_u8(dst.as_mut_ptr().add(i), vqsubq_u8(s, delta_vec));
                i += 16;
            }
        }
    }

    if i < len {
        super::scalar::brightness_sub(&src[i..len], &mut dst[i..len], delta, channels);
    }
}

/// Grayscale conversion for interleaved RGB pixels using ARM NEON (ITU-R BT.601 exact 16-bit fixed-point).
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn grayscale_rgb(src: &[u8], dst: &mut [u8]) {
    let num_pixels = src.len() / 3;
    let dst_len = dst.len().min(num_pixels);
    let mut px = 0;

    unsafe {
        // Process 16 pixels (48 source bytes -> 16 destination bytes) per iteration
        while px + 16 <= dst_len {
            let src_ptr = src.as_ptr().add(px * 3);
            let rgb = vld3q_u8(src_ptr);

            // Low 8 pixels
            let r_low_u16 = vmovl_u8(vget_low_u8(rgb.0));
            let g_low_u16 = vmovl_u8(vget_low_u8(rgb.1));
            let b_low_u16 = vmovl_u8(vget_low_u8(rgb.2));

            let mut acc0 = vdupq_n_u32(32768);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(r_low_u16), 19595);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(g_low_u16), 38470);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(b_low_u16), 7471);
            let gray0 = vshrn_n_u32(acc0, 16);

            let mut acc1 = vdupq_n_u32(32768);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(r_low_u16), 19595);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(g_low_u16), 38470);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(b_low_u16), 7471);
            let gray1 = vshrn_n_u32(acc1, 16);

            let gray_low_8 = vmovn_u16(vcombine_u16(gray0, gray1));

            // High 8 pixels
            let r_high_u16 = vmovl_u8(vget_high_u8(rgb.0));
            let g_high_u16 = vmovl_u8(vget_high_u8(rgb.1));
            let b_high_u16 = vmovl_u8(vget_high_u8(rgb.2));

            let mut acc2 = vdupq_n_u32(32768);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(r_high_u16), 19595);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(g_high_u16), 38470);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(b_high_u16), 7471);
            let gray2 = vshrn_n_u32(acc2, 16);

            let mut acc3 = vdupq_n_u32(32768);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(r_high_u16), 19595);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(g_high_u16), 38470);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(b_high_u16), 7471);
            let gray3 = vshrn_n_u32(acc3, 16);

            let gray_high_8 = vmovn_u16(vcombine_u16(gray2, gray3));

            let gray_16 = vcombine_u8(gray_low_8, gray_high_8);
            vst1q_u8(dst.as_mut_ptr().add(px), gray_16);
            px += 16;
        }
    }

    if px < dst_len {
        super::scalar::grayscale_rgb(&src[px * 3..], &mut dst[px..dst_len]);
    }
}

/// Grayscale conversion for interleaved RGBA pixels using ARM NEON (ITU-R BT.601 exact 16-bit fixed-point).
#[cfg(target_arch = "aarch64")]
#[inline]
pub fn grayscale_rgba(src: &[u8], dst: &mut [u8]) {
    let num_pixels = src.len() / 4;
    let dst_len = dst.len().min(num_pixels);
    let mut px = 0;

    unsafe {
        while px + 16 <= dst_len {
            let src_ptr = src.as_ptr().add(px * 4);
            let rgba = vld4q_u8(src_ptr);

            let r_low_u16 = vmovl_u8(vget_low_u8(rgba.0));
            let g_low_u16 = vmovl_u8(vget_low_u8(rgba.1));
            let b_low_u16 = vmovl_u8(vget_low_u8(rgba.2));

            let mut acc0 = vdupq_n_u32(32768);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(r_low_u16), 19595);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(g_low_u16), 38470);
            acc0 = vmlal_n_u16(acc0, vget_low_u16(b_low_u16), 7471);
            let gray0 = vshrn_n_u32(acc0, 16);

            let mut acc1 = vdupq_n_u32(32768);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(r_low_u16), 19595);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(g_low_u16), 38470);
            acc1 = vmlal_n_u16(acc1, vget_high_u16(b_low_u16), 7471);
            let gray1 = vshrn_n_u32(acc1, 16);

            let gray_low_8 = vmovn_u16(vcombine_u16(gray0, gray1));

            let r_high_u16 = vmovl_u8(vget_high_u8(rgba.0));
            let g_high_u16 = vmovl_u8(vget_high_u8(rgba.1));
            let b_high_u16 = vmovl_u8(vget_high_u8(rgba.2));

            let mut acc2 = vdupq_n_u32(32768);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(r_high_u16), 19595);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(g_high_u16), 38470);
            acc2 = vmlal_n_u16(acc2, vget_low_u16(b_high_u16), 7471);
            let gray2 = vshrn_n_u32(acc2, 16);

            let mut acc3 = vdupq_n_u32(32768);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(r_high_u16), 19595);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(g_high_u16), 38470);
            acc3 = vmlal_n_u16(acc3, vget_high_u16(b_high_u16), 7471);
            let gray3 = vshrn_n_u32(acc3, 16);

            let gray_high_8 = vmovn_u16(vcombine_u16(gray2, gray3));

            let gray_16 = vcombine_u8(gray_low_8, gray_high_8);
            vst1q_u8(dst.as_mut_ptr().add(px), gray_16);
            px += 16;
        }
    }

    if px < dst_len {
        super::scalar::grayscale_rgba(&src[px * 4..], &mut dst[px..dst_len]);
    }
}

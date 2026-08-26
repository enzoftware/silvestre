//! Portable scalar implementation of pixel manipulation kernels.
//!
//! Serves as the reference implementation, verification oracle, and fallback
//! for platforms/targets without SIMD capabilities.

/// Inverts color channels in `src` and writes to `dst`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn invert(src: &[u8], dst: &mut [u8], channels: usize) {
    let len = src.len().min(dst.len());
    if channels == 4 {
        let (src_chunks, src_rem) = src[..len].as_chunks::<4>();
        let (dst_chunks, dst_rem) = dst[..len].as_chunks_mut::<4>();
        for (s, d) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
            d[0] = 255 - s[0];
            d[1] = 255 - s[1];
            d[2] = 255 - s[2];
            d[3] = s[3]; // Preserve alpha
        }
        for (s, d) in src_rem.iter().zip(dst_rem.iter_mut()) {
            *d = 255 - *s;
        }
    } else {
        for (s, d) in src[..len].iter().zip(dst[..len].iter_mut()) {
            *d = 255 - *s;
        }
    }
}

/// Saturated brightness addition: `dst = clamp(src + delta, 0, 255)`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn brightness_add(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    if channels == 4 {
        let (src_chunks, src_rem) = src[..len].as_chunks::<4>();
        let (dst_chunks, dst_rem) = dst[..len].as_chunks_mut::<4>();
        for (s, d) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
            d[0] = s[0].saturating_add(delta);
            d[1] = s[1].saturating_add(delta);
            d[2] = s[2].saturating_add(delta);
            d[3] = s[3]; // Preserve alpha
        }
        for (s, d) in src_rem.iter().zip(dst_rem.iter_mut()) {
            *d = s.saturating_add(delta);
        }
    } else {
        for (s, d) in src[..len].iter().zip(dst[..len].iter_mut()) {
            *d = s.saturating_add(delta);
        }
    }
}

/// Saturated brightness subtraction: `dst = clamp(src - delta, 0, 255)`.
///
/// For `channels == 4` (RGBA), the alpha channel (every 4th byte) is preserved.
#[inline]
pub fn brightness_sub(src: &[u8], dst: &mut [u8], delta: u8, channels: usize) {
    let len = src.len().min(dst.len());
    if channels == 4 {
        let (src_chunks, src_rem) = src[..len].as_chunks::<4>();
        let (dst_chunks, dst_rem) = dst[..len].as_chunks_mut::<4>();
        for (s, d) in src_chunks.iter().zip(dst_chunks.iter_mut()) {
            d[0] = s[0].saturating_sub(delta);
            d[1] = s[1].saturating_sub(delta);
            d[2] = s[2].saturating_sub(delta);
            d[3] = s[3]; // Preserve alpha
        }
        for (s, d) in src_rem.iter().zip(dst_rem.iter_mut()) {
            *d = s.saturating_sub(delta);
        }
    } else {
        for (s, d) in src[..len].iter().zip(dst[..len].iter_mut()) {
            *d = s.saturating_sub(delta);
        }
    }
}

/// Converts interleaved RGB pixels to single-channel Grayscale using ITU-R BT.601.
#[inline]
pub fn grayscale_rgb(src: &[u8], dst: &mut [u8]) {
    let (chunks, _) = src.as_chunks::<3>();
    for (&[r, g, b], d) in chunks.iter().zip(dst.iter_mut()) {
        let lum = (0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        *d = lum;
    }
}

/// Converts interleaved RGBA pixels to single-channel Grayscale (alpha discarded).
#[inline]
pub fn grayscale_rgba(src: &[u8], dst: &mut [u8]) {
    let (chunks, _) = src.as_chunks::<4>();
    for (&[r, g, b, _], d) in chunks.iter().zip(dst.iter_mut()) {
        let lum = (0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b))
            .round()
            .clamp(0.0, 255.0) as u8;
        *d = lum;
    }
}

//! # Splitting the Colour Endpoints
//!
//! Each BC1 texture has 2 colour endpoints, `color0` and `color1`.
//! It is sometimes beneficial to separate these, i.e. store them separately.
//!
//! **File Size**: This technique reduces file size around 78% of the time.
//!
//! Take our optimized layout from earlier:
//!
//! ```ignore
//! +-------+-------+-------+
//! |C0  C1 |C0  C1 |C0  C1 |
//! +-------+-------+-------+
//! ```
//!
//! We can split the colour endpoints
//!
//! ```ignore
//! +-------+-------+ +-------+-------+
//! |C0  C0 |C0  C0 | |C1  C1 |C1  C1 |
//! +-------+-------+ +-------+-------+
//! ```

pub(crate) mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;

#[cfg(feature = "bench")]
pub mod bench;

#[cfg(test)]
pub mod tests;

use crate::color_565::Color565;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_color_endpoints_x86(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if crate::cpu_detect::has_avx512vbmi() {
            avx512::avx512_impl(colors, colors_out, colors_len_bytes);
            return;
        }

        if crate::cpu_detect::has_avx2() {
            avx2::avx2_shuf_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }

        if crate::cpu_detect::has_sse2() {
            sse2::sse2_shuf_unroll2_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_impl(colors, colors_out, colors_len_bytes);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::avx2_shuf_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::sse2_shuf_unroll2_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32(colors, colors_out, colors_len_bytes)
}

/// Splits the colour endpoints using the best known implementation for the current CPU.
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 4
///
/// # Remarks
///
/// For performance it's recommended that colors and colors_out are 32-byte aligned.
/// As this method may use a SIMD implementation.
#[inline]
pub unsafe fn split_color_endpoints(
    colors: *const Color565,
    colors_out: *mut Color565,
    colors_len_bytes: usize,
) {
    // Debug assert: colors_len_bytes must be at least 4 and a multiple of 4
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes.is_multiple_of(4),
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_color_endpoints_x86(colors as *const u8, colors_out as *mut u8, colors_len_bytes)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32(colors as *const u8, colors_out as *mut u8, colors_len_bytes)
    }
}

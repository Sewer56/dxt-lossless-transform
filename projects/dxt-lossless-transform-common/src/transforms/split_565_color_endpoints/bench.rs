//! Benchmark functions re-exported for external benchmarks.
//!
//! This module re-exposes internal benchmark functions that were changed to `pub(crate)` visibility
//! so that external benchmarks can still access them when the `bench` feature is enabled.
#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]

/// Re-exported benchmark functions for split_565_color_endpoints
pub mod split_565_color_endpoints {
    /// Split 565 color endpoints benchmark functions
    // Portable implementations
    pub unsafe fn u32(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable32::u32(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u32_with_separate_endpoints(
        max_input_ptr: *const u32,
        input: *const u32,
        output0: *mut u16,
        output1: *mut u16,
    ) {
        crate::transforms::split_565_color_endpoints::portable32::u32_with_separate_endpoints(
            max_input_ptr,
            input,
            output0,
            output1,
        )
    }

    pub unsafe fn u64(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable64::u64(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_unroll_2(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable64::u64_unroll_2(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_unroll_4(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable64::u64_unroll_4(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_unroll_8(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable64::u64_unroll_8(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_mix(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::portable64::u64_mix(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_mix_unroll_2(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::portable64::u64_mix_unroll_2(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_mix_unroll_4(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::portable64::u64_mix_unroll_4(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    pub unsafe fn u64_mix_unroll_8(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::portable64::u64_mix_unroll_8(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    // SSE2 implementations
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn sse2_shift_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::sse2::sse2_shift_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn sse2_shuf_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::sse2::sse2_shuf_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn sse2_shuf_unroll2_impl_asm(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::sse2::sse2_shuf_unroll2_impl_asm(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn sse2_shuf_unroll2_impl(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::sse2::sse2_shuf_unroll2_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    // SSSE3 implementations
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn ssse3_pshufb_unroll2_impl(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::ssse3::ssse3_pshufb_unroll2_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn ssse3_pshufb_unroll4_impl(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::ssse3::ssse3_pshufb_unroll4_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    // AVX2 implementations
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx2_shuf_impl_asm(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::avx2::avx2_shuf_impl_asm(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx2_shuf_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::avx2::avx2_shuf_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx2_shuf_impl_unroll_2(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::avx2::avx2_shuf_impl_unroll_2(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    // AVX512 implementations
    #[cfg(feature = "nightly")]
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx512_impl_unroll2(
        colors: *const u8,
        colors_out: *mut u8,
        colors_len_bytes: usize,
    ) {
        crate::transforms::split_565_color_endpoints::avx512::avx512_impl_unroll2(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }

    #[cfg(feature = "nightly")]
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx512_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
        crate::transforms::split_565_color_endpoints::avx512::avx512_impl(
            colors,
            colors_out,
            colors_len_bytes,
        )
    }
}

use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;
pub(crate) mod generic;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

/// Split BC3 blocks from standard interleaved format to separate alpha/color/index format,
/// applying YCoCg-R decorrelation to color endpoints.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes
/// - `len` must be divisible by 16
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    decorrelation_mode: YCoCgVariant,
) {
    debug_assert!(len.is_multiple_of(16));

    // BC3 output layout: alpha_endpoints(2) + alpha_indices(6) + colors(4) + color_indices(4) = 16 bytes per block
    let alpha_endpoints_ptr = output_ptr as *mut u16;
    let alpha_indices_ptr = output_ptr.add(len / 8) as *mut u16; // len/16 * 2 = len/8
    let colors_ptr = output_ptr.add(len / 2) as *mut u32; // len/16 * 8 = len/2
    let color_indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32; // len/16 * 12 = 3*len/4
    let num_blocks = len / 16;

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_with_decorrelate_x86(
            input_ptr,
            alpha_endpoints_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            num_blocks,
            decorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        generic::transform_with_decorrelate_generic(
            input_ptr,
            alpha_endpoints_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            num_blocks,
            decorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[allow(dead_code)]
#[inline(always)]
unsafe fn transform_with_decorrelate_x86(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        if has_avx2() {
            avx2::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        if has_sse2() {
            sse2::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "avx2") {
            avx2::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "sse2") {
            sse2::transform_with_decorrelate(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    // Fallback to generic implementation
    generic::transform_with_decorrelate_generic(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
        decorrelation_mode,
    );
}

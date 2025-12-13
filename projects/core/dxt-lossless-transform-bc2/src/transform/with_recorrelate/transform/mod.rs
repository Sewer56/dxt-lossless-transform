use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;
pub(crate) mod generic;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

/// Split BC2 blocks from standard interleaved format to separate alpha/color/index format,
/// applying YCoCg-R decorrelation to color endpoints.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes
/// - `len` must be divisible by 16
#[inline]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    decorrelation_mode: YCoCgVariant,
) {
    debug_assert!(len.is_multiple_of(16));

    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;
    let num_blocks = len / 16;

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_with_decorrelate_x86(
            input_ptr,
            alphas_ptr,
            colors_ptr,
            indices_ptr,
            num_blocks,
            decorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        generic::transform_with_decorrelate_generic(
            input_ptr,
            alphas_ptr,
            colors_ptr,
            indices_ptr,
            num_blocks,
            decorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_with_decorrelate_x86(
    input_ptr: *const u8,
    alphas_out: *mut u64,
    colors_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if has_avx512bw() {
            avx512::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        if has_avx2() {
            avx2::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        if has_sse2() {
            sse2::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512bw") {
            avx512::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "avx2") {
            avx2::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "sse2") {
            sse2::transform_with_decorrelate(
                input_ptr,
                alphas_out,
                colors_out,
                indices_out,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    // Fallback to generic implementation
    generic::transform_with_decorrelate_generic(
        input_ptr,
        alphas_out,
        colors_out,
        indices_out,
        num_blocks,
        decorrelation_mode,
    );
}

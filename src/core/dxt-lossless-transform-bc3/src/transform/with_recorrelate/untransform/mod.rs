use dxt_lossless_transform_common::color_565::YCoCgVariant;

pub(crate) mod generic;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512vbmi;

#[cfg(target_arch = "x86")]
mod avx512vbmi_32;

#[cfg(target_arch = "x86_64")]
mod avx512vbmi_64;

/// Transform BC3 data from separated alpha/color/index format back to standard interleaved format
/// while applying YCoCg recorrelation.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
#[inline(always)]
pub(crate) unsafe fn untransform_with_recorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    recorrelation_mode: YCoCgVariant,
) {
    debug_assert!(len.is_multiple_of(16));

    // Setup pointers to separated data sections
    let alpha_endpoints_ptr = input_ptr as *const u16;
    let alpha_indices_ptr = input_ptr.add(len / 8) as *const u16; // len/16 * 2 = len/8
    let colors_ptr = input_ptr.add(len / 2) as *const u32; // len/16 * 8 = len/2
    let color_indices_ptr = input_ptr.add(len / 2 + len / 4) as *const u32; // len/16 * 12 = 3*len/4
    let num_blocks = len / 16; // Each BC3 block is 16 bytes

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        untransform_with_recorrelate_x86(
            alpha_endpoints_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            output_ptr,
            num_blocks,
            recorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::untransform_with_recorrelate_generic(
            alpha_endpoints_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            output_ptr,
            num_blocks,
            recorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
unsafe fn untransform_with_recorrelate_x86(
    alpha_endpoints_ptr: *const u16,
    alpha_indices_ptr: *const u16,
    colors_ptr: *const u32,
    color_indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if has_avx512vbmi() {
            avx512vbmi::untransform_with_recorrelate(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
            return;
        }

        if has_sse2() {
            sse2::untransform_with_recorrelate(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512vbmi") {
            avx512vbmi::untransform_with_recorrelate(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_recorrelate(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
            return;
        }
    }

    // Fallback to portable implementation
    generic::untransform_with_recorrelate_generic(
        alpha_endpoints_ptr,
        alpha_indices_ptr,
        colors_ptr,
        color_indices_ptr,
        output_ptr,
        num_blocks,
        recorrelation_mode,
    );
}

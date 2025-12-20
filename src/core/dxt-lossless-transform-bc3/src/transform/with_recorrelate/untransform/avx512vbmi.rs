#![allow(missing_docs)]

use core::hint::unreachable_unchecked;

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// AVX512VBMI implementation of BC3 untransform with YCoCg-R recorrelation.
///
/// # Safety
///
/// - alpha_endpoints_in must be valid for reads of num_blocks * 2 bytes
/// - alpha_indices_in must be valid for reads of num_blocks * 6 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - color_indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        super::avx512vbmi_64::untransform_recorr_64::<1>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        super::avx512vbmi_32::untransform_recorr_32::<1>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

unsafe fn untransform_recorr_var2(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        super::avx512vbmi_64::untransform_recorr_64::<2>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        super::avx512vbmi_32::untransform_recorr_32::<2>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

unsafe fn untransform_recorr_var3(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        super::avx512vbmi_64::untransform_recorr_64::<3>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        super::avx512vbmi_32::untransform_recorr_32::<3>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 64)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 64)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 64)]
    fn avx512vbmi_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512vbmi() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "AVX512VBMI");
    }
}

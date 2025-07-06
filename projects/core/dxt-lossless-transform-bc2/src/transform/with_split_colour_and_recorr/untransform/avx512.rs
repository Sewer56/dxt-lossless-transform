use crate::transform::with_split_colour_and_recorr::untransform::generic;
//#[cfg(target_arch = "x86")]
//use core::arch::x86::*;
//#[cfg(target_arch = "x86_64")]
//use core::arch::x86_64::*;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// AVX512 implementation for split-colour and recorrelate untransform for BC2.
/// Currently delegates to generic implementation.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f,avx512bw")]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    // For now, delegate to generic implementation
    // TODO: Implement optimized AVX512 version
    generic::untransform_with_split_colour_and_recorr(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
        recorrelation_mode,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn avx512_untransform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_avx512f() || !has_avx512bw() {
            return;
        }

        // Generic fallback processes 16 bytes per iteration (* 2 / 16 == 2)
        run_split_colour_and_recorr_untransform_roundtrip_test(
            untransform_with_split_colour_and_recorr,
            variant,
            2,
            "AVX512",
        );
    }
}

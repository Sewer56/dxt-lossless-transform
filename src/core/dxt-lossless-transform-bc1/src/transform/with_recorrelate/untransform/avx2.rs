#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx2::{
    recorrelate_ycocg_r_var1_avx2, recorrelate_ycocg_r_var2_avx2, recorrelate_ycocg_r_var3_avx2,
};

pub(crate) unsafe fn untransform_with_recorrelate(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => {
            // This should be unreachable based on the calling context
            unreachable_unchecked()
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_in, indices_in, output_ptr, num_blocks)
}

#[target_feature(enable = "avx2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // === Main Vectorized Loop ===
    // Process 16 blocks at a time using AVX2 SIMD instructions (unroll 2)
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let colors_end = colors_in.add(vectorized_blocks);

    // Main SIMD processing loop - handles 16 blocks per iteration (unroll 2)
    while colors_in < colors_end {
        // Load colors and indices (32 bytes each, containing 8 blocks worth of data)
        let colors_0 = _mm256_loadu_si256(colors_in as *const __m256i);
        let colors_1 = _mm256_loadu_si256(colors_in.add(8) as *const __m256i);

        let indices_0 = _mm256_loadu_si256(indices_in as *const __m256i);
        let indices_1 = _mm256_loadu_si256(indices_in.add(8) as *const __m256i);

        // Apply permutation to get proper ordering (equivalent to vpermq with 0xD8)
        // 0xD8 = 11 01 10 00 = [0, 2, 1, 3] which reorders 64-bit elements
        let colors_perm_0 = _mm256_permute4x64_epi64(colors_0, 0xD8);
        let colors_perm_1 = _mm256_permute4x64_epi64(colors_1, 0xD8);
        let indices_perm_0 = _mm256_permute4x64_epi64(indices_0, 0xD8);
        let indices_perm_1 = _mm256_permute4x64_epi64(indices_1, 0xD8);

        // Apply recorrelation to the colors based on the variant
        let recorrelated_colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx2(colors_perm_0),
            2 => recorrelate_ycocg_r_var2_avx2(colors_perm_0),
            3 => recorrelate_ycocg_r_var3_avx2(colors_perm_0),
            _ => unreachable_unchecked(),
        };
        let recorrelated_colors_1 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx2(colors_perm_1),
            2 => recorrelate_ycocg_r_var2_avx2(colors_perm_1),
            3 => recorrelate_ycocg_r_var3_avx2(colors_perm_1),
            _ => unreachable_unchecked(),
        };

        // Interleave colors and indices using unpack operations
        // This is equivalent to vpunpckldq and vpunpckhdq from the assembly
        let output_0 = _mm256_unpacklo_epi32(recorrelated_colors_0, indices_perm_0);
        let output_1 = _mm256_unpackhi_epi32(recorrelated_colors_0, indices_perm_0);
        let output_2 = _mm256_unpacklo_epi32(recorrelated_colors_1, indices_perm_1);
        let output_3 = _mm256_unpackhi_epi32(recorrelated_colors_1, indices_perm_1);

        // Store results (each __m256i contains 8 BC1 blocks worth of data)
        _mm256_storeu_si256(output_ptr as *mut __m256i, output_0);
        _mm256_storeu_si256(output_ptr.add(32) as *mut __m256i, output_1);
        _mm256_storeu_si256(output_ptr.add(64) as *mut __m256i, output_2);
        _mm256_storeu_si256(output_ptr.add(96) as *mut __m256i, output_3);

        colors_in = colors_in.add(16);
        indices_in = indices_in.add(16);
        output_ptr = output_ptr.add(128);
    }

    // === Scalar Fallback for Remaining Blocks ===
    // Handle any remaining blocks that couldn't be processed in the vectorized loop
    // (when num_blocks is not a multiple of 16) using generic implementation
    let remaining_count = num_blocks - vectorized_blocks;
    let variant = match VARIANT {
        1 => YCoCgVariant::Variant1,
        2 => YCoCgVariant::Variant2,
        3 => YCoCgVariant::Variant3,
        _ => unreachable_unchecked(),
    };
    super::generic::untransform_with_recorrelate_generic(
        colors_in,
        indices_in,
        output_ptr,
        remaining_count,
        variant,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use dxt_lossless_transform_common::cpu_detect::has_avx2;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: WithRecorrelateUntransformFn,
        #[case] decorr_variant: YCoCgVariant,
    ) {
        if !has_avx2() {
            return;
        }

        run_with_recorrelate_untransform_unaligned_test(
            function,
            decorr_variant,
            "untransform_with_recorrelate (avx2)",
            32, // 128 bytes tested per main loop iteration (* 2 / 8 == 32)
        );
    }
}

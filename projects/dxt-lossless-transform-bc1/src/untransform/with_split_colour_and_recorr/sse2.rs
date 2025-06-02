#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::*;

pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::None => unreachable_unchecked(),
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<1>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

unsafe fn untransform_recorr_var2(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<2>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

unsafe fn untransform_recorr_var3(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<3>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

#[target_feature(enable = "sse2")]
#[allow(clippy::identity_op)]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 8 blocks (64 bytes) at a time with SSE2
    let aligned_count = block_count - (block_count % 8);
    let color0_ptr_aligned_end = color0_ptr.add(aligned_count);

    if aligned_count > 0 {
        while color0_ptr < color0_ptr_aligned_end {
            let color0s = _mm_loadu_si128(color0_ptr as *const __m128i);
            color0_ptr = color0_ptr.add(8);

            let color1s = _mm_loadu_si128(color1_ptr as *const __m128i);
            color1_ptr = color1_ptr.add(8);

            let indices_0 = _mm_loadu_si128(indices_ptr as *const __m128i);
            let indices_1 = _mm_loadu_si128(indices_ptr.add(4) as *const __m128i);
            indices_ptr = indices_ptr.add(8);

            // Apply YCoCg-R recorrelation to the colors using the specified variant
            let (recorr_color0s, recorr_color1s) = match VARIANT {
                1 => (
                    recorrelate_ycocg_r_var1_sse2(color0s),
                    recorrelate_ycocg_r_var1_sse2(color1s),
                ),
                2 => (
                    recorrelate_ycocg_r_var2_sse2(color0s),
                    recorrelate_ycocg_r_var2_sse2(color1s),
                ),
                3 => (
                    recorrelate_ycocg_r_var3_sse2(color0s),
                    recorrelate_ycocg_r_var3_sse2(color1s),
                ),
                _ => unreachable_unchecked(),
            };

            // Mix the colours back into their c0+c1 pairs
            let colors_0 = _mm_unpacklo_epi16(recorr_color0s, recorr_color1s);
            let colors_1 = _mm_unpackhi_epi16(recorr_color0s, recorr_color1s);

            // Re-combine the colors and indices into the BC1 block format
            let blocks_0 = _mm_unpacklo_epi32(colors_0, indices_0);
            let blocks_1 = _mm_unpackhi_epi32(colors_0, indices_0);
            let blocks_2 = _mm_unpacklo_epi32(colors_1, indices_1);
            let blocks_3 = _mm_unpackhi_epi32(colors_1, indices_1);

            _mm_storeu_si128(output_ptr as *mut __m128i, blocks_0);
            _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, blocks_1);
            _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, blocks_2);
            _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, blocks_3);

            // Advance output pointer
            output_ptr = output_ptr.add(64);
        }
    }

    // Process any remaining blocks (less than 8) using scalar code
    let remaining_count = block_count - aligned_count;
    if remaining_count > 0 {
        let mut remaining_color0_ptr = color0_ptr;
        let mut remaining_color1_ptr = color1_ptr;
        let mut remaining_indices_ptr = indices_ptr;
        let mut remaining_output_ptr = output_ptr;

        let remaining_color0_ptr_end = remaining_color0_ptr.add(remaining_count);

        while remaining_color0_ptr < remaining_color0_ptr_end {
            // Read the split color values
            let color0 = remaining_color0_ptr.read_unaligned();
            let color1 = remaining_color1_ptr.read_unaligned();
            let indices = remaining_indices_ptr.read_unaligned();

            // Apply YCoCg-R recorrelation to the color pair using the specified variant
            let color0_obj = Color565::from_raw(color0);
            let color1_obj = Color565::from_raw(color1);
            let (recorr_color0, recorr_color1) = match VARIANT {
                1 => (
                    color0_obj.recorrelate_ycocg_r_var1(),
                    color1_obj.recorrelate_ycocg_r_var1(),
                ),
                2 => (
                    color0_obj.recorrelate_ycocg_r_var2(),
                    color1_obj.recorrelate_ycocg_r_var2(),
                ),
                3 => (
                    color0_obj.recorrelate_ycocg_r_var3(),
                    color1_obj.recorrelate_ycocg_r_var3(),
                ),
                _ => unreachable_unchecked(),
            };

            // Write BC1 block format: [color0: u16, color1: u16, indices: u32]
            (remaining_output_ptr as *mut u16).write_unaligned(recorr_color0.raw_value());
            (remaining_output_ptr.add(2) as *mut u16).write_unaligned(recorr_color1.raw_value());
            (remaining_output_ptr.add(4) as *mut u32).write_unaligned(indices);

            // Advance all pointers
            remaining_color0_ptr = remaining_color0_ptr.add(1);
            remaining_color1_ptr = remaining_color1_ptr.add(1);
            remaining_indices_ptr = remaining_indices_ptr.add(1);
            remaining_output_ptr = remaining_output_ptr.add(8);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::has_sse2;
    use rstest::rstest;

    use super::untransform_with_split_colour_and_recorr;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(#[case] decorr_variant: YCoCgVariant) {
        if !has_sse2() {
            return;
        }

        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            let mut work = vec![0u8; original.len()];
            unsafe {
                transform_bc1(
                    original.as_ptr(),
                    transformed.as_mut_ptr(),
                    work.as_mut_ptr(),
                    original.len(),
                    Bc1TransformDetails {
                        color_normalization_mode: ColorNormalizationMode::None,
                        decorrelation_mode: decorr_variant,
                        split_colour_endpoints: true,
                    },
                );
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);
            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                untransform_with_split_colour_and_recorr(
                    transformed_unaligned.as_ptr().add(1) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                    decorr_variant,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "untransform_with_split_colour_and_recorr (sse2, unaligned)",
                num_blocks,
            );
        }
    }
}

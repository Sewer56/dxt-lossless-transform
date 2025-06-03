use crate::split_blocks::unsplit::unsplit_block_with_separate_pointers;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::{
    allocate::allocate_align_64,
    color_565::{Color565, YCoCgVariant},
};

pub(crate) unsafe fn untransform_with_split_colour_and_recorr_generic(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    // Note(sewer): I can't get good generic codegen for this at the moment,
    //              on x86, the codegen is poor, moving values in and out of SIMD registers
    //              for the recorrelation step.
    //
    //              I'm guessing the same will be the case on aarch64 (don't have a high end aarch64
    //              machine to test on). So I've opted for another solution for the time being, that
    //              uses 2 functions (which do get correctly optimized) and an intermediate work buffer.
    //
    //              This will make the function limited by memory bandwidth, due to extra copy.
    //              For x86 at least, I got custom intrinsic functions, to overcome this.

    // Allocating here has some overhead, so we'll delegate to the slower solution if under 512 bytes.
    // 64 blocks is 512 bytes
    if block_count >= 64 {
        let mut work_alloc =
            allocate_align_64(block_count * 8).expect("Failed to allocate work buffer");
        let work_ptr = work_alloc.as_mut_ptr();

        // Recorrelate colours into work area, doing the unsplit in the same process.
        Color565::recorrelate_ycocg_r_ptr_split(
            color0_ptr as *mut Color565,
            color1_ptr as *mut Color565,
            work_ptr as *mut Color565,
            block_count * 2, // 2 colour endpoints per block.
            recorrelation_mode,
        );

        // Now unsplit the colours, placing them into the final buffer
        unsplit_block_with_separate_pointers(
            work_ptr as *const u32,
            indices_ptr,
            output_ptr,
            block_count * 8,
        );
        return;
    }

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
pub(crate) unsafe fn untransform_recorr_var1(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<1>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

pub(crate) unsafe fn untransform_recorr_var2(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<2>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

pub(crate) unsafe fn untransform_recorr_var3(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<3>(color0_ptr, color1_ptr, indices_ptr, output_ptr, block_count)
}

unsafe fn untransform_recorr<const VARIANT: u8>(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    // Initialize pointers for iteration
    let mut color0_ptr = color0_ptr;
    let mut color1_ptr = color1_ptr;
    let mut indices_ptr = indices_ptr;
    let mut output_ptr = output_ptr;

    for _ in 0..block_count {
        // Read the correlated colors and apply recorrelation using the specified variant
        let color0_obj = Color565::from_raw(color0_ptr.read_unaligned());
        let color1_obj = Color565::from_raw(color1_ptr.read_unaligned());
        let (recorrelated_color0, recorrelated_color1) = match VARIANT {
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

        // Read the indices
        let indices = indices_ptr.read_unaligned();

        // Write the BC1 block directly: color0 (2 bytes) + color1 (2 bytes) + indices (4 bytes)
        // Colors are stored in little-endian format as u16 values
        (output_ptr as *mut u16).write_unaligned(recorrelated_color0.raw_value());
        (output_ptr.add(2) as *mut u16).write_unaligned(recorrelated_color1.raw_value());
        (output_ptr.add(4) as *mut u32).write_unaligned(indices);

        // Advance pointers
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
        output_ptr = output_ptr.add(8); // 8 bytes per BC1 block
    }
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::with_split_colour_and_recorr::generic::untransform_with_split_colour_and_recorr_generic;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::has_sse2;
    use rstest::rstest;

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
                untransform_with_split_colour_and_recorr_generic(
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
                "untransform_with_split_colour_and_recorr (generic, unaligned)",
                num_blocks,
            );
        }
    }
}

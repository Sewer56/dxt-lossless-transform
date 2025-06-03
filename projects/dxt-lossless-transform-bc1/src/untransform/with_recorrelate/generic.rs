use core::hint::unreachable_unchecked;
use core::ptr::read_unaligned;
use core::ptr::write_unaligned;
use dxt_lossless_transform_common::color_565::Color565;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

pub(crate) unsafe fn untransform_with_recorrelate_generic(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

unsafe fn untransform_recorr<const VARIANT: u8>(
    mut colors_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    unsafe {
        for _ in 0..num_blocks {
            // Read both values first (better instruction scheduling)
            let color_raw = read_unaligned(colors_ptr);
            let index_value = read_unaligned(indices_ptr);

            colors_ptr = colors_ptr.add(1);
            indices_ptr = indices_ptr.add(1);

            // Extract both [`Color565`] values from the u32
            let color0 = Color565::from_raw(color_raw as u16);
            let color1 = Color565::from_raw((color_raw >> 16) as u16);

            // Apply recorrelation to both colors based on the variant
            let (recorr_color0, recorr_color1) = match VARIANT {
                1 => (
                    color0.recorrelate_ycocg_r_var1(),
                    color1.recorrelate_ycocg_r_var1(),
                ),
                2 => (
                    color0.recorrelate_ycocg_r_var2(),
                    color1.recorrelate_ycocg_r_var2(),
                ),
                3 => (
                    color0.recorrelate_ycocg_r_var3(),
                    color1.recorrelate_ycocg_r_var3(),
                ),
                _ => unreachable_unchecked(),
            };

            // Pack both recorrelated colors back into u32
            let recorrelated_colors =
                (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

            // Write both values together
            write_unaligned(output_ptr as *mut u32, recorrelated_colors);
            write_unaligned(output_ptr.add(4) as *mut u32, index_value);

            output_ptr = output_ptr.add(8);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::with_recorrelate::generic::*;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use rstest::rstest;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: unsafe fn(*const u32, *const u32, *mut u8, usize) -> (),
        #[case] decorr_variant: YCoCgVariant,
    ) {
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
                        split_colour_endpoints: false,
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
                function(
                    transformed_unaligned.as_ptr().add(1) as *const u32,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "unaligned untransform with recorrelation (generic)",
                num_blocks,
            );
        }
    }
}

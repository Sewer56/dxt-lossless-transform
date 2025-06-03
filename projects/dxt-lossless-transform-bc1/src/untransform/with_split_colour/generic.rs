pub(crate) unsafe fn untransform_with_split_colour(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    // Fallback implementation
    unsafe {
        // Initialize pointers
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        // Calculate end pointer for color0
        let color0_ptr_end = color0_ptr.add(block_count);

        while color0_ptr < color0_ptr_end {
            // Read the split color values
            let color0 = color0_ptr.read_unaligned();
            let color1 = color1_ptr.read_unaligned();
            let indices = indices_ptr.read_unaligned();

            // Write BC1 block format: [color0: u16, color1: u16, indices: u32]
            // Convert to bytes and write directly
            (output_ptr as *mut u16).write_unaligned(color0);
            (output_ptr.add(2) as *mut u16).write_unaligned(color1);
            (output_ptr.add(4) as *mut u32).write_unaligned(indices);

            // Advance all pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::with_split_colour::generic::untransform_with_split_colour;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;

    #[test]
    fn can_untransform_unaligned() {
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
                        decorrelation_mode: YCoCgVariant::None,
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
                untransform_with_split_colour(
                    transformed_unaligned.as_ptr().add(1) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "untransform_with_split_colour (generic, unaligned)",
                num_blocks,
            );
        }
    }
}

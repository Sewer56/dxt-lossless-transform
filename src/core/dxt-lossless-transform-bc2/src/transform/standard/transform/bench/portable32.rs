/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    let mut input_ptr = input_ptr as *mut u8;
    let max_ptr = input_ptr.add(len);
    let max_aligned_input = input_ptr.add(len.saturating_sub(32));

    while input_ptr < max_aligned_input {
        // First block
        let alpha_value1 = (input_ptr as *const u64).read_unaligned();
        let color_value1 = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value1 = (input_ptr.add(12) as *const u32).read_unaligned();

        // Second block
        let alpha_value2 = (input_ptr.add(16) as *const u64).read_unaligned();
        let color_value2 = (input_ptr.add(24) as *const u32).read_unaligned();
        let index_value2 = (input_ptr.add(28) as *const u32).read_unaligned();
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(32);

        // Store first block
        alphas_ptr.write_unaligned(alpha_value1);
        colours_ptr.write_unaligned(color_value1);
        indices_ptr.write_unaligned(index_value1);

        // Store second block
        alphas_ptr.add(1).write_unaligned(alpha_value2);
        colours_ptr.add(1).write_unaligned(color_value2);
        indices_ptr.add(1).write_unaligned(index_value2);

        // Advance pointers
        alphas_ptr = alphas_ptr.add(2);
        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = (input_ptr as *const u64).read_unaligned();
        let color_value = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value = (input_ptr.add(12) as *const u32).read_unaligned();
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        alphas_ptr.write_unaligned(alpha_value);
        colours_ptr.write_unaligned(color_value);
        indices_ptr.write_unaligned(index_value);

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    let mut input_ptr = input_ptr as *mut u8;
    let max_ptr = input_ptr.add(len);
    let max_aligned_input = input_ptr.add(len.saturating_sub(64));

    while input_ptr < max_aligned_input {
        // First block
        let alpha_value1 = (input_ptr as *const u64).read_unaligned();
        let color_value1 = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value1 = (input_ptr.add(12) as *const u32).read_unaligned();

        // Second block
        let alpha_value2 = (input_ptr.add(16) as *const u64).read_unaligned();
        let color_value2 = (input_ptr.add(24) as *const u32).read_unaligned();
        let index_value2 = (input_ptr.add(28) as *const u32).read_unaligned();

        // Third block
        let alpha_value3 = (input_ptr.add(32) as *const u64).read_unaligned();
        let color_value3 = (input_ptr.add(40) as *const u32).read_unaligned();
        let index_value3 = (input_ptr.add(44) as *const u32).read_unaligned();

        // Fourth block
        let alpha_value4 = (input_ptr.add(48) as *const u64).read_unaligned();
        let color_value4 = (input_ptr.add(56) as *const u32).read_unaligned();
        let index_value4 = (input_ptr.add(60) as *const u32).read_unaligned();
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(64);

        // Store all blocks
        alphas_ptr.write_unaligned(alpha_value1);
        colours_ptr.write_unaligned(color_value1);
        indices_ptr.write_unaligned(index_value1);

        alphas_ptr.add(1).write_unaligned(alpha_value2);
        colours_ptr.add(1).write_unaligned(color_value2);
        indices_ptr.add(1).write_unaligned(index_value2);

        alphas_ptr.add(2).write_unaligned(alpha_value3);
        colours_ptr.add(2).write_unaligned(color_value3);
        indices_ptr.add(2).write_unaligned(index_value3);

        alphas_ptr.add(3).write_unaligned(alpha_value4);
        colours_ptr.add(3).write_unaligned(color_value4);
        indices_ptr.add(3).write_unaligned(index_value4);

        // Advance pointers
        alphas_ptr = alphas_ptr.add(4);
        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = (input_ptr as *const u64).read_unaligned();
        let color_value = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value = (input_ptr.add(12) as *const u32).read_unaligned();
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        alphas_ptr.write_unaligned(alpha_value);
        colours_ptr.write_unaligned(color_value);
        indices_ptr.write_unaligned(index_value);

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_unroll_2, "u32 unroll_2", 4)]
    #[case(u32_unroll_4, "u32 unroll_4", 8)]
    fn test_portable32_unaligned(
        #[case] permute_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        run_standard_transform_unaligned_test(permute_fn, max_blocks, impl_name);
    }
}

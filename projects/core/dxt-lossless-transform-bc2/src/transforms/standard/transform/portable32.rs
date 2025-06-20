/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Split output into color and index sections
    // Note(sewer): Compiler will split u64 into 2 u32 registers, so from our perspective
    // whether we go for u32 or u64 is irrelevant
    let alphas_ptr = output_ptr as *mut u64;
    let colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    u32_with_separate_pointers(input_ptr, alphas_ptr, colours_ptr, indices_ptr, len);
}

/// Inner function that processes the data with separate pointers for each component
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len/2 bytes
/// - colours_ptr must be valid for writes of len/4 bytes
/// - indices_ptr must be valid for writes of len/4 bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u32_with_separate_pointers(
    input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

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
    #[case(u32, "u32 no-unroll")]
    fn test_portable32_unaligned(#[case] permute_fn: StandardTransformFn, #[case] impl_name: &str) {
        run_standard_transform_unaligned_test(permute_fn, 2, impl_name);
    }
}

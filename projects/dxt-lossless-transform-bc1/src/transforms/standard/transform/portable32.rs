use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Split output into color and index sections
    let colours_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    u32_with_separate_pointers(input_ptr, colours_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colours_ptr must be valid for writes of len/2 bytes
/// - indices_ptr must be valid for writes of len/2 bytes
/// - len must be divisible by 8
#[inline]
pub(crate) unsafe fn u32_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colours_out: *mut u32,
    mut indices_out: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let color_value = read_unaligned(input_ptr as *const u32);
        let index_value = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store colours and indices to their respective halves
        write_unaligned(colours_out, color_value);
        write_unaligned(indices_out, index_value);

        colours_out = colours_out.add(1);
        indices_out = indices_out.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32, "u32 no-unroll")]
    fn portable32_transform_roundtrip(#[case] permute_fn: TransformFn, #[case] impl_name: &str) {
        run_standard_transform_roundtrip_test(permute_fn, 512, impl_name);
    }
}

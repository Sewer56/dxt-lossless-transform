use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Get pointers to the color and index sections
    let colours_ptr = input_ptr as *const u32;
    let indices_ptr = input_ptr.add(len / 2) as *const u32;

    u32_detransform_with_separate_pointers(colours_ptr, indices_ptr, output_ptr, len);
}

/// # Safety
///
/// - colours_ptr must be valid for reads of len/2 bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
#[inline]
pub(crate) unsafe fn u32_detransform_with_separate_pointers(
    mut colours_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    // Calculate end pointer for the indices section
    let max_indices_ptr = indices_ptr.add(len / 8);

    while indices_ptr < max_indices_ptr {
        // Read color and index values
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32_detransform, "u32")]
    fn test_portable32_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        run_standard_untransform_aligned_test(detransform_fn, 512, impl_name);
    }

    #[rstest]
    #[case(u32_detransform, "u32")]
    fn test_portable32_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        run_standard_untransform_unaligned_test(detransform_fn, 512, impl_name);
    }
}

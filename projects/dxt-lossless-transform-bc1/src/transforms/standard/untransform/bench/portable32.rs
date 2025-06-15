use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(16 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        indices_ptr = indices_ptr.add(2);

        // Load colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        colours_ptr = colours_ptr.add(2);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        output_ptr = output_ptr.add(16);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_detransform_unroll_4(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(32 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        let index_value3 = read_unaligned(indices_ptr.add(2));
        let index_value4 = read_unaligned(indices_ptr.add(3));
        indices_ptr = indices_ptr.add(4);

        // Load all colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        let color_value3 = read_unaligned(colours_ptr.add(2));
        let color_value4 = read_unaligned(colours_ptr.add(3));
        colours_ptr = colours_ptr.add(4);

        // Write all values to output in order
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        write_unaligned(output_ptr.add(16) as *mut u32, color_value3);
        write_unaligned(output_ptr.add(20) as *mut u32, index_value3);

        write_unaligned(output_ptr.add(24) as *mut u32, color_value4);
        write_unaligned(output_ptr.add(28) as *mut u32, index_value4);

        output_ptr = output_ptr.add(32);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_detransform_unroll_8(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(64 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        let index_value3 = read_unaligned(indices_ptr.add(2));
        let index_value4 = read_unaligned(indices_ptr.add(3));
        let index_value5 = read_unaligned(indices_ptr.add(4));
        let index_value6 = read_unaligned(indices_ptr.add(5));
        let index_value7 = read_unaligned(indices_ptr.add(6));
        let index_value8 = read_unaligned(indices_ptr.add(7));
        indices_ptr = indices_ptr.add(8);

        // Load all colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        let color_value3 = read_unaligned(colours_ptr.add(2));
        let color_value4 = read_unaligned(colours_ptr.add(3));
        let color_value5 = read_unaligned(colours_ptr.add(4));
        let color_value6 = read_unaligned(colours_ptr.add(5));
        let color_value7 = read_unaligned(colours_ptr.add(6));
        let color_value8 = read_unaligned(colours_ptr.add(7));
        colours_ptr = colours_ptr.add(8);

        // Write all values to output in order
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        write_unaligned(output_ptr.add(16) as *mut u32, color_value3);
        write_unaligned(output_ptr.add(20) as *mut u32, index_value3);

        write_unaligned(output_ptr.add(24) as *mut u32, color_value4);
        write_unaligned(output_ptr.add(28) as *mut u32, index_value4);

        write_unaligned(output_ptr.add(32) as *mut u32, color_value5);
        write_unaligned(output_ptr.add(36) as *mut u32, index_value5);

        write_unaligned(output_ptr.add(40) as *mut u32, color_value6);
        write_unaligned(output_ptr.add(44) as *mut u32, index_value6);

        write_unaligned(output_ptr.add(48) as *mut u32, color_value7);
        write_unaligned(output_ptr.add(52) as *mut u32, index_value7);

        write_unaligned(output_ptr.add(56) as *mut u32, color_value8);
        write_unaligned(output_ptr.add(60) as *mut u32, index_value8);

        output_ptr = output_ptr.add(64);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
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

    #[rstest]
    #[case(u32_detransform_unroll_2, "unroll_2")]
    #[case(u32_detransform_unroll_4, "unroll_4")]
    #[case(u32_detransform_unroll_8, "unroll_8")]
    fn test_portable32_unaligned(
        #[case] detransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
    ) {
        run_standard_untransform_unaligned_test(detransform_fn, 512, impl_name);
    }
}

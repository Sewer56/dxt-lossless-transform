/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Get pointers to the color, index sections, and end of data.
    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_input = input_ptr.add(len) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_input {
        // Read color and index values
        let index_value = *indices_ptr;
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = *colours_ptr;
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        *(output_ptr as *mut u32) = color_value;
        *(output_ptr.add(4) as *mut u32) = index_value;

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (for unroll 2)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_input = input_ptr.add(len) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_input {
        // Load indices first and advance pointer immediately
        let index_value1 = *indices_ptr;
        let index_value2 = *indices_ptr.add(1);
        indices_ptr = indices_ptr.add(2);

        // Load colors after indices
        let color_value1 = *colours_ptr;
        let color_value2 = *colours_ptr.add(1);
        colours_ptr = colours_ptr.add(2);

        // Write interleaved values to output
        *(output_ptr as *mut u32) = color_value1;
        *(output_ptr.add(4) as *mut u32) = index_value1;

        *(output_ptr.add(8) as *mut u32) = color_value2;
        *(output_ptr.add(12) as *mut u32) = index_value2;

        output_ptr = output_ptr.add(16);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32 (for unroll 4)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_input = input_ptr.add(len) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = *indices_ptr;
        let index_value2 = *indices_ptr.add(1);
        let index_value3 = *indices_ptr.add(2);
        let index_value4 = *indices_ptr.add(3);
        indices_ptr = indices_ptr.add(4);

        // Load all colors after indices
        let color_value1 = *colours_ptr;
        let color_value2 = *colours_ptr.add(1);
        let color_value3 = *colours_ptr.add(2);
        let color_value4 = *colours_ptr.add(3);
        colours_ptr = colours_ptr.add(4);

        // Write all values to output in order
        *(output_ptr as *mut u32) = color_value1;
        *(output_ptr.add(4) as *mut u32) = index_value1;

        *(output_ptr.add(8) as *mut u32) = color_value2;
        *(output_ptr.add(12) as *mut u32) = index_value2;

        *(output_ptr.add(16) as *mut u32) = color_value3;
        *(output_ptr.add(20) as *mut u32) = index_value3;

        *(output_ptr.add(24) as *mut u32) = color_value4;
        *(output_ptr.add(28) as *mut u32) = index_value4;

        output_ptr = output_ptr.add(32);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (for unroll 8)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_input = input_ptr.add(len) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = *indices_ptr;
        let index_value2 = *indices_ptr.add(1);
        let index_value3 = *indices_ptr.add(2);
        let index_value4 = *indices_ptr.add(3);
        let index_value5 = *indices_ptr.add(4);
        let index_value6 = *indices_ptr.add(5);
        let index_value7 = *indices_ptr.add(6);
        let index_value8 = *indices_ptr.add(7);
        indices_ptr = indices_ptr.add(8);

        // Load all colors after indices
        let color_value1 = *colours_ptr;
        let color_value2 = *colours_ptr.add(1);
        let color_value3 = *colours_ptr.add(2);
        let color_value4 = *colours_ptr.add(3);
        let color_value5 = *colours_ptr.add(4);
        let color_value6 = *colours_ptr.add(5);
        let color_value7 = *colours_ptr.add(6);
        let color_value8 = *colours_ptr.add(7);
        colours_ptr = colours_ptr.add(8);

        // Write all values to output in order
        *(output_ptr as *mut u32) = color_value1;
        *(output_ptr.add(4) as *mut u32) = index_value1;

        *(output_ptr.add(8) as *mut u32) = color_value2;
        *(output_ptr.add(12) as *mut u32) = index_value2;

        *(output_ptr.add(16) as *mut u32) = color_value3;
        *(output_ptr.add(20) as *mut u32) = index_value3;

        *(output_ptr.add(24) as *mut u32) = color_value4;
        *(output_ptr.add(28) as *mut u32) = index_value4;

        *(output_ptr.add(32) as *mut u32) = color_value5;
        *(output_ptr.add(36) as *mut u32) = index_value5;

        *(output_ptr.add(40) as *mut u32) = color_value6;
        *(output_ptr.add(44) as *mut u32) = index_value6;

        *(output_ptr.add(48) as *mut u32) = color_value7;
        *(output_ptr.add(52) as *mut u32) = index_value7;

        *(output_ptr.add(56) as *mut u32) = color_value8;
        *(output_ptr.add(60) as *mut u32) = index_value8;

        output_ptr = output_ptr.add(64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bc1::split_blocks::tests::generate_bc1_test_data;
    use crate::bc1::split_blocks::u32;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::unroll_2(
        TestCase {
            name: "no_unroll",
            func: u32_detransform,
            min_blocks: 1,
            many_blocks: 1024,
        }
    )]
    #[case::unroll_2(
        TestCase {
            name: "unroll_2",
            func: u32_detransform_unroll_2,
            min_blocks: 2,
            many_blocks: 1024,
        }
    )]
    #[case::unroll_4(
        TestCase {
            name: "unroll_4",
            func: u32_detransform_unroll_4,
            min_blocks: 4,
            many_blocks: 1024,
        }
    )]
    #[case::unroll_8(
        TestCase {
            name: "unroll_8",
            func: u32_detransform_unroll_8,
            min_blocks: 8,
            many_blocks: 1024,
        }
    )]
    fn test_detransform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let original = generate_bc1_test_data(num_blocks);
        let mut transformed = vec![0u8; original.len()];
        let mut reconstructed = vec![0u8; original.len()];

        unsafe {
            u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            (test_case.func)(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "{} detransform failed to reconstruct original data for {} blocks",
            test_case.name,
            num_blocks
        );
    }
}

use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        asm!(
            // Calculate indices pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}", // indices_ptr = colors_ptr + len / 2
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}", // end = indices_ptr + len / 2

            ".p2align 5",
            "2:",
            // Load colors and indices (16 bytes each)
            "movdqu xmm0, [{colors_ptr}]",    // colors
            "add {colors_ptr}, 16",
            "movdqu xmm1, [{indices_ptr}]",   // indices
            "add {indices_ptr}, 16",

            // Interleave the 32-bit values
            "movaps xmm2, xmm0",
            "unpcklps xmm0, xmm1",    // Low half: color0,index0,color1,index1
            "unpckhps xmm2, xmm1",    // High half: color2,index2,color3,index3

            // Store the results
            "movdqu [{dst_ptr}], xmm0",
            "movdqu [{dst_ptr} + 16], xmm2",
            "add {dst_ptr}, 32",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = out(reg) _,
            dst_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = out(reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::dxt1::transform::tests::generate_dxt1_test_data;
    use crate::raw::transform;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::sse(
        TestCase {
            name: "sse",
            func: unpck_detransform,
            min_blocks: 4,
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
        let original = generate_dxt1_test_data(num_blocks);
        let mut transformed = vec![0u8; original.len()];
        let mut reconstructed = vec![0u8; original.len()];

        unsafe {
            transform::u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
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

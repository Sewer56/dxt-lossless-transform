use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len must be divisible by 32
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        let mut indices_ptr = input_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            ".p2align 5",
            "2:",
            // Load colors and indices (16 bytes each)
            "movdqu {xmm0}, [{colors_ptr}]",    // colors
            "add {colors_ptr}, 16",
            "movdqu {xmm1}, [{indices_ptr}]",   // indices
            "add {indices_ptr}, 16",

            // Interleave the 32-bit values
            "movaps {xmm2}, {xmm0}",
            "unpcklps {xmm0}, {xmm1}",    // Low half: color0,index0,color1,index1
            "unpckhps {xmm2}, {xmm1}",    // High half: color2,index2,color3,index3

            // Store the results
            "movdqu [{dst_ptr}], {xmm0}",
            "movdqu [{dst_ptr} + 16], {xmm2}",
            "add {dst_ptr}, 32",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = inout(reg) indices_ptr,
            dst_ptr = inout(reg) output_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len must be divisible by 64 (processes 2 blocks per iteration)
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut indices_ptr = input_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            ".p2align 5",
            "2:",
            // Load all colors and indices (32 bytes each)
            "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
            "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
            "add {colors_ptr}, 32",
            "movdqu {xmm1}, [{indices_ptr}]",     // indices 0
            "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
            "add {indices_ptr}, 32",

            // Save copies for high parts
            "movaps {xmm2}, {xmm0}", // colors 0 copy
            "movaps {xmm5}, {xmm3}", // colors 1 copy

            // Unpack all blocks
            "punpckldq {xmm0}, {xmm1}", // color0,index0,color1,index1
            "punpckldq {xmm3}, {xmm4}", // color4,index4,color5,index5
            "punpckhdq {xmm2}, {xmm1}", // color2,index2,color3,index3
            "punpckhdq {xmm5}, {xmm4}", // color6,index6,color7,index7

            // Store all results
            "movdqu [{dst_ptr}], {xmm0}",
            "movdqu [{dst_ptr} + 16], {xmm2}",
            "movdqu [{dst_ptr} + 32], {xmm3}",
            "movdqu [{dst_ptr} + 48], {xmm5}",
            "add {dst_ptr}, 64",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = inout(reg) indices_ptr,
            dst_ptr = inout(reg) output_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len must be divisible by 128 (processes 4 blocks per iteration)
#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut indices_ptr = input_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            ".p2align 5",
            "2:",
            // Load all colors (64 bytes)
            "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
            "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
            "movdqu {xmm6}, [{colors_ptr} + 32]", // colors 2
            "movdqu {xmm9}, [{colors_ptr} + 48]", // colors 3
            "add {colors_ptr}, 64",

            // Load all indices (64 bytes)
            "movdqu {xmm1}, [{indices_ptr}]",      // indices 0
            "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
            "movdqu {xmm7}, [{indices_ptr} + 32]", // indices 2
            "movdqu {xmm10}, [{indices_ptr} + 48]", // indices 3
            "add {indices_ptr}, 64",

            // Save copies for high parts
            "movaps {xmm2}, {xmm0}",  // colors 0 copy
            "movaps {xmm5}, {xmm3}",  // colors 1 copy
            "movaps {xmm8}, {xmm6}",  // colors 2 copy
            "movaps {xmm11}, {xmm9}", // colors 3 copy

            // Unpack all blocks
            "unpcklps {xmm0}, {xmm1}",
            "unpckhps {xmm2}, {xmm1}",
            "unpcklps {xmm3}, {xmm4}",
            "unpckhps {xmm5}, {xmm4}",
            "unpcklps {xmm6}, {xmm7}",
            "unpckhps {xmm8}, {xmm7}",
            "unpcklps {xmm9}, {xmm10}",
            "unpckhps {xmm11}, {xmm10}",

            // Store all results
            "movdqu [{dst_ptr}], {xmm0}",
            "movdqu [{dst_ptr} + 16], {xmm2}",
            "movdqu [{dst_ptr} + 32], {xmm3}",
            "movdqu [{dst_ptr} + 48], {xmm5}",
            "movdqu [{dst_ptr} + 64], {xmm6}",
            "movdqu [{dst_ptr} + 80], {xmm8}",
            "movdqu [{dst_ptr} + 96], {xmm9}",
            "movdqu [{dst_ptr} + 112], {xmm11}",
            "add {dst_ptr}, 128",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = inout(reg) indices_ptr,
            dst_ptr = inout(reg) output_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            xmm6 = out(xmm_reg) _,
            xmm7 = out(xmm_reg) _,
            xmm8 = out(xmm_reg) _,
            xmm9 = out(xmm_reg) _,
            xmm10 = out(xmm_reg) _,
            xmm11 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bc1::split_colours::tests::generate_bc1_test_data;
    use crate::bc1::split_colours::u32;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::unpck(TestCase {
        name: "unpck",
        func: unpck_detransform,
        min_blocks: 4,
        many_blocks: 1024,
    })]
    #[case::unpck_unroll_2(TestCase {
        name: "unpck_unroll_2",
        func: unpck_detransform_unroll_2,
        min_blocks: 8,  // Needs at least 8 blocks due to 64-byte alignment requirement
        many_blocks: 1024,
    })]
    #[cfg_attr(target_arch = "x86_64", case::unpck_unroll_4(TestCase {
        name: "unpck_unroll_4",
        func: unpck_detransform_unroll_4,
        min_blocks: 16, // Needs at least 16 blocks due to 128-byte alignment requirement
        many_blocks: 1024,
    }))]
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

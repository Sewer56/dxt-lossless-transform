use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len must be divisible by 64
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[no_mangle]
pub unsafe fn shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut colors_ptr = input_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        let alpha_ptr_end = colors_ptr;

        asm!(
            ".p2align 5",
            "2:",

            // Load components
            "movdqu {xmm0}, [{alpha_ptr}]",         // First alpha block
            "movdqu {xmm1}, [{alpha_ptr} + 16]",    // Second alpha block
            "add {alpha_ptr}, 32",
            "movdqu {xmm2}, [{colors_ptr}]",        // Colors
            "add {colors_ptr}, 16",
            "movdqu {xmm3}, [{indices_ptr}]",       // Indices
            "add {indices_ptr}, 16",

            // Current:
            // xmm0: [A0  - A15]
            // xmm1: [A16 - A31]
            // xmm2: [C0  - C15]
            // xmm3: [I0  - I15]

            // Target:
            // 0       -       7 |   08       -       15
            // block0: [A00 - A07] | [C00 - C03] [I00 - I03]
            // block1: [A08 - A15] | [C04 - C07] [I04 - I07]
            // block2: [A16 - A23] | [C08 - C11] [I08 - I11]
            // block3: [A24 - A31] | [C12 - C15] [I12 - I15]

            // Let's get [C00 - C03] [I00 - I03] ... inside XMM6
            // Let's get [C08 - C11] [I08 - I11] ... inside XMM7
            "movaps {xmm6}, {xmm2}",
            "movaps {xmm7}, {xmm2}",
            "punpckldq {xmm6}, {xmm3}", // Interleave 32-bit elements. We did it!
            "punpckhdq {xmm7}, {xmm3}", // Interleave 32-bit elements. We did it!
            // xmm6: [C00 - C03] [I00 - I03] [C04 - C07] [I04 - I07]
            // xmm7: [C08 - C11] [I08 - I11] [C12 - C15] [I12 - I15]

            // We're gonna now export results to remaining xmm registers
            // block0 = xmm0
            // block1 = xmm4
            // block2 = xmm1
            // block3 = xmm5

            // Interleave bottom 64 bits of XMM0 with bottom XMM6 to get block0.
            "movaps {xmm4}, {xmm0}",
            "movaps {xmm5}, {xmm1}",

            "punpcklqdq {xmm0}, {xmm6}", // block0
            "punpcklqdq {xmm1}, {xmm7}", // block2
            "punpckhqdq {xmm4}, {xmm6}", // block1
            "punpckhqdq {xmm5}, {xmm7}", // block3

            // Store results
            "movdqu [{output_ptr}], {xmm0}",
            "movdqu [{output_ptr} + 16], {xmm4}",
            "movdqu [{output_ptr} + 32], {xmm1}",
            "movdqu [{output_ptr} + 48], {xmm5}",
            "add {output_ptr}, 64",

            // Loop until done
            "cmp {alpha_ptr}, {alpha_ptr_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_end = in(reg) alpha_ptr_end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            xmm6 = out(xmm_reg) _,
            xmm7 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc2::transform::tests::generate_bc2_test_data;
    use crate::raw::bc2::transform::u32;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::shuffle(TestCase {
        name: "shuffle",
        func: shuffle,
        min_blocks: 4,
        many_blocks: 1024,
    })]
    fn test_detransform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let original = generate_bc2_test_data(num_blocks);
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

use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 128
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[no_mangle]
pub unsafe fn avx2_shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut colors_ptr = input_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        let alpha_ptr_end = colors_ptr;

        asm!(
            ".p2align 5",
            "2:",

            // Based on SSE solution, SSE version may be easier to read.
            // Load components
            "vmovdqu {ymm0}, [{alpha_ptr}]",         // First alpha block
            "vmovdqu {ymm1}, [{alpha_ptr} + 32]",    // Second alpha block
            "add {alpha_ptr}, 64",
            "vmovdqu {ymm2}, [{colors_ptr}]",        // Colors
            "add {colors_ptr}, 32",
            "vmovdqu {ymm3}, [{indices_ptr}]",       // Indices
            "add {indices_ptr}, 32",

            // Current:
            // ymm0: [A0  - A31]
            // ymm1: [A32 - A63]
            // ymm2: [C0  - C31]
            // ymm3: [I0  - I31]

            // Let's get [C00 - C03] [I00 - I03] ... inside YMM4
            // Let's get [C08 - C11] [I08 - I11] ... inside YMM5
            "vpunpckldq {ymm4}, {ymm2}, {ymm3}", // Interleave 32-bit elements. We did it!
            "vpunpckhdq {ymm5}, {ymm2}, {ymm3}", // Interleave 32-bit elements. We did it!

            // ymm4 {
            //     [-128, -127, -126, -125],
            //     [-64, -63, -62, -61],
            //     [-124, -123, -122, -121],
            //     [-60, -59, -58, -57],
            //
            //     [-112, -111, -110, -109],
            //     [-48, -47, -46, -45],
            //     [-108, -107, -106, -105],
            //     [-44, -43, -42, -41]
            // }

            // ymm5 {
            //     [-120, -119, -118, -117],
            //     [-56, -55, -54, -53],
            //     [-116, -115, -114, -113],
            //     [-52, -51, -50, -49],
            //
            //     [-104, -103, -102, -101],
            //     [-40, -39, -38, -37],
            //     [-100, -99, -98, -97],
            //     [-36, -35, -34, -33]
            // }
            "vperm2i128 {ymm6}, {ymm4}, {ymm5}, 0x20", // First halves (0x20 = 0b00100000)
            "vperm2i128 {ymm7}, {ymm4}, {ymm5}, 0x31", // Second halves (0x31 = 0b00110001)
            // Now ymm6, ymm7 are in proper order.

            // ymm6 {
            //     [-128, -127, -126, -125],
            //     [-64, -63, -62, -61],
            //     [-124, -123, -122, -121],
            //     [-60, -59, -58, -57],
            //
            //     [-120, -119, -118, -117],
            //     [-56, -55, -54, -53],
            //     [-116, -115, -114, -113],
            //     [-52, -51, -50, -49],
            // }

            // ymm7 {
            //     [-112, -111, -110, -109],
            //     [-48, -47, -46, -45],
            //     [-108, -107, -106, -105],
            //     [-44, -43, -42, -41]
            //
            //     [-104, -103, -102, -101],
            //     [-40, -39, -38, -37],
            //     [-100, -99, -98, -97],
            //     [-36, -35, -34, -33]
            // }

            // We're gonna now export results to remaining xmm registers
            // Interleave bottom 64 bits of XMM0 with bottom XMM6 to get block0.
            "vpunpckhqdq {ymm4}, {ymm0}, {ymm6}", // block2+3
            "vpunpckhqdq {ymm5}, {ymm1}, {ymm7}", // block6+7
            "vpunpcklqdq {ymm0}, {ymm0}, {ymm6}", // block0+1
            "vpunpcklqdq {ymm1}, {ymm1}, {ymm7}", // block4+5

            // We got the items, but because we were shifting data across multiple lanes,
            // the upper half of the registers is out of order. We need to restore order.
            // ymm0: {
            //   [0, 1, 2, 3, 4, 5, 6, 7], [-128, -127, -126, -125, -64, -63, -62, -61],
            //   [16, 17, 18, 19, 20, 21, 22, 23], [-120, -119, -118, -117, -56, -55, -54, -53]
            // }
            // ymm4: {
            //   [8, 9, 10, 11, 12, 13, 14, 15], [-124, -123, -122, -121, -60, -59, -58, -57],
            //   [24, 25, 26, 27, 28, 29, 30, 31], [-116, -115, -114, -113, -52, -51, -50, -49]
            // }
            // ymm1: {
            //   [32, 33, 34, 35, 36, 37, 38, 39], [-112, -111, -110, -109, -48, -47, -46, -45],
            //   [48, 49, 50, 51, 52, 53, 54, 55], [-104, -103, -102, -101, -40, -39, -38, -37]
            // }
            // ymm5: {
            //   [40, 41, 42, 43, 44, 45, 46, 47], [-108, -107, -106, -105, -44, -43, -42, -41],
            //   [56, 57, 58, 59, 60, 61, 62, 63], [-100, -99, -98, -97, -36, -35, -34, -33]
            // }

            "vperm2i128 {ymm2}, {ymm0}, {ymm4}, 0x20", // First halves (0x20 = 0b00100000)
            "vperm2i128 {ymm3}, {ymm0}, {ymm4}, 0x31", // Second halves (0x31 = 0b00110001)
            "vperm2i128 {ymm6}, {ymm1}, {ymm5}, 0x20", // First halves
            "vperm2i128 {ymm7}, {ymm1}, {ymm5}, 0x31", // Second halves

            // Store results
            "vmovdqu [{output_ptr}], {ymm2}",
            "vmovdqu [{output_ptr} + 32], {ymm3}",
            "vmovdqu [{output_ptr} + 64], {ymm6}",
            "vmovdqu [{output_ptr} + 96], {ymm7}",
            "add {output_ptr}, 128",

            // Loop until done
            "cmp {alpha_ptr}, {alpha_ptr_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_end = in(reg) alpha_ptr_end,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
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
    #[case::avx2_shuffle(TestCase {
        name: "avx2_shuffle",
        func: avx2_shuffle,
        min_blocks: 8,
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

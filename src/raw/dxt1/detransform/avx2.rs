use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 64 (processes 32 bytes of input/output per iteration)
#[allow(unused_assignments)]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        asm!(
            // Calculate indices pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}", // indices_ptr = colors_ptr + len / 2
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}", // end = indices_ptr + len / 2

            ".p2align 5",
            "2:",
            // Load colors and indices (32 bytes each)
            "vmovdqu {ymm0}, [{colors_ptr}]",    // colors, [c0 c1 c2 c3 c4 c5 c6 c7]
            "add {colors_ptr}, 32",
            "vmovdqu {ymm1}, [{indices_ptr}]",   // indices [i0 i1 i2 i3 i4 i5 i6 i7]
            "add {indices_ptr}, 32",

            // Unpack and interleave the values, reusing ymm0/ymm1
            "vunpckhps {ymm2}, {ymm0}, {ymm1}",  // [c2 i2 c3 i3 | c6 i6 c7 i7]
            "vunpcklps {ymm0}, {ymm0}, {ymm1}",  // [c0 i0 c1 i1 | c4 i4 c5 i5]

            // Permute to get final layout
            "vperm2f128 {ymm1}, {ymm0}, {ymm2}, 32",  // [c0 i0 c1 i1 | c2 i2 c3 i3]
            "vperm2f128 {ymm0}, {ymm0}, {ymm2}, 49",  // [c4 i4 c5 i5 | c6 i6 c7 i7]

            // Store results
            "vmovdqu [{dst_ptr}], {ymm1}",
            "vmovdqu [{dst_ptr} + 32], {ymm0}",
            "add {dst_ptr}, 64",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = out(reg) _,
            dst_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = out(reg) _,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 128 (processes 64 bytes of input/output per iteration)
#[allow(unused_assignments)]
pub unsafe fn unpck_detransform_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}",

            ".p2align 5",
            "2:",
            // Load first set - colors and indices (32 bytes each)
            "vmovdqu {ymm0}, [{colors_ptr}]",        // [c0 c1 c2 c3 c4 c5 c6 c7]
            "vmovdqu {ymm3}, [{colors_ptr} + 32]",   // [c8 c9 c10 c11 c12 c13 c14 c15]
            "add {colors_ptr}, 64",

            "vmovdqu {ymm1}, [{indices_ptr}]",       // [i0 i1 i2 i3 i4 i5 i6 i7]
            "vmovdqu {ymm4}, [{indices_ptr} + 32]",  // [i8 i9 i10 i11 i12 i13 i14 i15]
            "add {indices_ptr}, 64",

            // Unpack first 32-byte chunk
            "vunpckhps {ymm2}, {ymm0}, {ymm1}",      // [c2 i2 c3 i3 | c6 i6 c7 i7]
            "vunpcklps {ymm0}, {ymm0}, {ymm1}",      // [c0 i0 c1 i1 | c4 i4 c5 i5]

            // Unpack second 32-byte chunk
            "vunpckhps {ymm5}, {ymm3}, {ymm4}",      // [c10 i10 c11 i11 | c14 i14 c15 i15]
            "vunpcklps {ymm3}, {ymm3}, {ymm4}",      // [c8 i8 c9 i9 | c12 i12 c13 i13]

            // Permute first chunk
            "vperm2f128 {ymm1}, {ymm0}, {ymm2}, 32", // [c0 i0 c1 i1 | c2 i2 c3 i3]
            "vperm2f128 {ymm0}, {ymm0}, {ymm2}, 49", // [c4 i4 c5 i5 | c6 i6 c7 i7]

            // Permute second chunk
            "vperm2f128 {ymm4}, {ymm3}, {ymm5}, 32", // [c8 i8 c9 i9 | c10 i10 c11 i11]
            "vperm2f128 {ymm3}, {ymm3}, {ymm5}, 49", // [c12 i12 c13 i13 | c14 i14 c15 i15]

            // Store all results
            "vmovdqu [{dst_ptr}], {ymm1}",
            "vmovdqu [{dst_ptr} + 32], {ymm0}",
            "vmovdqu [{dst_ptr} + 64], {ymm4}",
            "vmovdqu [{dst_ptr} + 96], {ymm3}",
            "add {dst_ptr}, 128",

            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = out(reg) _,
            dst_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = out(reg) _,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 256 (processes 128 bytes of input/output per iteration)
///
/// Note: This function requires x86_64 due to the number of registers used
#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
pub unsafe fn unpck_detransform_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 256 == 0);

    unsafe {
        asm!(
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}",
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}",

            ".p2align 5",
            "2:",
            // Load all colors (128 bytes total)
            "vmovdqu {ymm0}, [{colors_ptr}]",         // [c0-c7]
            "vmovdqu {ymm3}, [{colors_ptr} + 32]",    // [c8-c15]
            "vmovdqu {ymm6}, [{colors_ptr} + 64]",    // [c16-c23]
            "vmovdqu {ymm9}, [{colors_ptr} + 96]",    // [c24-c31]
            "add {colors_ptr}, 128",

            // Load all indices (128 bytes total)
            "vmovdqu {ymm1}, [{indices_ptr}]",        // [i0-i7]
            "vmovdqu {ymm4}, [{indices_ptr} + 32]",   // [i8-i15]
            "vmovdqu {ymm7}, [{indices_ptr} + 64]",   // [i16-i23]
            "vmovdqu {ymm10}, [{indices_ptr} + 96]",  // [i24-i31]
            "add {indices_ptr}, 128",

            // Unpack first 32-byte chunk
            "vunpckhps {ymm2}, {ymm0}, {ymm1}",
            "vunpcklps {ymm0}, {ymm0}, {ymm1}",

            // Unpack second 32-byte chunk
            "vunpckhps {ymm5}, {ymm3}, {ymm4}",
            "vunpcklps {ymm3}, {ymm3}, {ymm4}",

            // Unpack third 32-byte chunk
            "vunpckhps {ymm8}, {ymm6}, {ymm7}",
            "vunpcklps {ymm6}, {ymm6}, {ymm7}",

            // Unpack fourth 32-byte chunk
            "vunpckhps {ymm11}, {ymm9}, {ymm10}",
            "vunpcklps {ymm9}, {ymm9}, {ymm10}",

            // Permute all chunks
            "vperm2f128 {ymm1}, {ymm0}, {ymm2}, 32",
            "vperm2f128 {ymm0}, {ymm0}, {ymm2}, 49",
            "vperm2f128 {ymm4}, {ymm3}, {ymm5}, 32",
            "vperm2f128 {ymm3}, {ymm3}, {ymm5}, 49",
            "vperm2f128 {ymm7}, {ymm6}, {ymm8}, 32",
            "vperm2f128 {ymm6}, {ymm6}, {ymm8}, 49",
            "vperm2f128 {ymm10}, {ymm9}, {ymm11}, 32",
            "vperm2f128 {ymm9}, {ymm9}, {ymm11}, 49",

            // Store all results
            "vmovdqu [{dst_ptr}], {ymm1}",
            "vmovdqu [{dst_ptr} + 32], {ymm0}",
            "vmovdqu [{dst_ptr} + 64], {ymm4}",
            "vmovdqu [{dst_ptr} + 96], {ymm3}",
            "vmovdqu [{dst_ptr} + 128], {ymm7}",
            "vmovdqu [{dst_ptr} + 160], {ymm6}",
            "vmovdqu [{dst_ptr} + 192], {ymm10}",
            "vmovdqu [{dst_ptr} + 224], {ymm9}",
            "add {dst_ptr}, 256",

            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = out(reg) _,
            dst_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = out(reg) _,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
            ymm8 = out(ymm_reg) _,
            ymm9 = out(ymm_reg) _,
            ymm10 = out(ymm_reg) _,
            ymm11 = out(ymm_reg) _,
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
    #[case::avx(TestCase {
        name: "avx",
        func: unpck_detransform,
        min_blocks: 8,  // 64-byte alignment requirement
        many_blocks: 1024,
    })]
    #[case::avx_unroll_2(TestCase {
        name: "avx_unroll_2",
        func: unpck_detransform_unroll_2,
        min_blocks: 16,  // 128-byte alignment requirement
        many_blocks: 1024,
    })]
    #[cfg_attr(target_arch = "x86_64", case::avx_unroll_4(TestCase {
        name: "avx_unroll_4",
        func: unpck_detransform_unroll_4,
        min_blocks: 32,  // 256-byte alignment requirement
        many_blocks: 1024,
    }))]
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

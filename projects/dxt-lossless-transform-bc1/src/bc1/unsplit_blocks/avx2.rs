use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 64 (processes 32 bytes of input/output per iteration)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permd_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
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

            // As we load, we pre-arrange the loaded data for interleaving.
            // ymm0 {
            //     [00, 01, 02, 03] [BLOCK0 COL] 0
            //     [04, 05, 06, 07] [BLOCK1 COL] 1
            //     [08, 09, 10, 11] [BLOCK2 COL] 2
            //     [12, 13, 14, 15] [BLOCK3 COL] 3
            //
            //     [16, 17, 18, 19] [BLOCK4 COL] 4
            //     [20, 21, 22, 23] [BLOCK5 COL] 5
            //     [24, 25, 26, 27] [BLOCK6 COL] 6
            //     [28, 29, 30, 31] [BLOCK7 COL] 7
            // }
            // ymm1 {
            //     [128, 129, 130, 131] [BLOCK0 IND] 0
            //     [132, 133, 134, 135] [BLOCK1 IND] 1
            //     [136, 137, 138, 139] [BLOCK2 IND] 2
            //     [140, 141, 142, 143] [BLOCK3 IND] 3
            //
            //     [144, 145, 146, 147] [BLOCK4 IND] 4
            //     [148, 149, 150, 151] [BLOCK5 IND] 5
            //     [152, 153, 154, 155] [BLOCK6 IND] 6
            //     [156, 157, 158, 159] [BLOCK7 IND] 7
            // }

            // We want to interleave the values such that `vunpcklps` and `vunpckhps` get us
            // the values in the order we want. If we do that on a raw read we will get:
            // vunpcklps {ymm3}, {ymm0}, {ymm1}
            // ymm3 {
            //     [000, 001, 002, 003] [BLOCK0 COL]
            //     [128, 129, 130, 131] [BLOCK0 IND]
            //     [004, 005, 006, 007] [BLOCK1 COL]
            //     [132, 133, 134, 135] [BLOCK1 IND]
            //
            //     [016, 017, 018, 019] [BLOCK4 COL]
            //     [144, 145, 146, 147] [BLOCK4 IND]
            //     [020, 021, 022, 023] [BLOCK5 COL]
            //     [148, 149, 150, 151] [BLOCK5 IND]
            // }

            // To improve performance, we shuffle to make this work 'right' at read time.
            // So we want the data to be read in this format.

            // ymm0 {
            //     [000, 001, 002, 003] [BLOCK0 COL] 0
            //     [004, 005, 006, 007]
            //     [016, 017, 018, 019] [BLOCK2 COL] 2
            //     [020, 021, 022, 023]
            //
            //     [008, 009, 010, 011] [BLOCK1 COL] 1
            //     [012, 013, 014, 015]
            //     [024, 025, 026, 027] [BLOCK3 COL] 3
            //     [028, 029, 030, 031]
            // }
            // ymm1 {
            //     [128, 129, 130, 131] [BLOCK0 IND] 0
            //     [132, 133, 134, 135]
            //     [144, 145, 146, 147] [BLOCK2 IND] 2
            //     [148, 149, 150, 151]
            //
            //     [136, 137, 138, 139] [BLOCK1 IND] 1
            //     [140, 141, 142, 143]
            //     [152, 153, 154, 155] [BLOCK3 IND] 3
            //     [156, 157, 158, 159]
            // }
            // In illustration above, I reinterpreted as '64-bit' items, since we're doing
            // a 64-bit shuffle.

            "vpermq {ymm0}, [{colors_ptr}], 0xD8", // 11 01 10 00 [i.e. 0 2 1 3]
            "add {colors_ptr}, 32",
            "vpermq {ymm1}, [{indices_ptr}], 0xD8", // 11 01 10 00 [i.e. 0 2 1 3]
            "add {indices_ptr}, 32",

            // Unpack and interleave the values, reusing ymm0/ymm1
            "vpunpckldq {ymm2}, {ymm0}, {ymm1}",  // [c0 i0 c1 i1 | c4 i4 c5 i5]
            "vpunpckhdq {ymm3}, {ymm0}, {ymm1}",  // [c2 i2 c3 i3 | c6 i6 c7 i7]

            // Store results
            "vmovdqu [{dst_ptr}], {ymm2}",
            "vmovdqu [{dst_ptr} + 32], {ymm3}",
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
            ymm3 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 64 (processes 32 bytes of input/output per iteration)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permd_detransform_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 128 == 0); // Must handle 128 bytes per iteration now (2x64)

    unsafe {
        // To understand this code, see non-unrolled version.
        asm!(
            // Calculate indices pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}", // indices_ptr = colors_ptr + len / 2
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}", // end = indices_ptr + len / 2

            ".p2align 5",
            "2:",
            // Load first set of colors and indices
            "vpermq {ymm0}, [{colors_ptr}], 0xD8",      // First colors block
            "vpermq {ymm1}, [{indices_ptr}], 0xD8",     // First indices block

            // Start loading second set while first set processes
            "vpermq {ymm4}, [{colors_ptr} + 32], 0xD8", // Second colors block
            "vpermq {ymm5}, [{indices_ptr} + 32], 0xD8", // Second indices block
            "add {colors_ptr}, 64",
            "add {indices_ptr}, 64",

            // Process first set
            "vpunpckldq {ymm2}, {ymm0}, {ymm1}",        // Interleave low parts of first set
            "vpunpckhdq {ymm3}, {ymm0}, {ymm1}",        // Interleave high parts of first set

            // Process second set
            "vpunpckldq {ymm6}, {ymm4}, {ymm5}",        // Interleave low parts of second set
            "vpunpckhdq {ymm7}, {ymm4}, {ymm5}",        // Interleave high parts of second set

            // Store results with some spacing to help with memory operations
            "vmovdqu [{dst_ptr}], {ymm2}",             // Store first low part
            "vmovdqu [{dst_ptr} + 32], {ymm3}",        // Store first high part
            "vmovdqu [{dst_ptr} + 64], {ymm6}",        // Store second low part
            "vmovdqu [{dst_ptr} + 96], {ymm7}",        // Store second high part

            // Update pointers for next iteration
            "add {dst_ptr}, 128",

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
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 256 (processes 64 bytes of input/output per iteration)
/// - x86_64 only
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
pub unsafe fn permd_detransform_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 256 == 0); // Must handle 256 bytes per iteration now (4x64)

    unsafe {
        asm!(
            // Calculate indices pointer
            "mov {indices_ptr}, {colors_ptr}",
            "add {indices_ptr}, {len_half}", // indices_ptr = colors_ptr + len / 2
            "mov {end}, {indices_ptr}",
            "add {end}, {len_half}", // end = indices_ptr + len / 2

            ".p2align 5",
            "2:",
            // Load all colors blocks first to get them in flight
            "vpermq {ymm0}, [{colors_ptr}], 0xD8",        // Colors block 1
            "vpermq {ymm4}, [{colors_ptr} + 32], 0xD8",   // Colors block 2
            "vpermq {ymm8}, [{colors_ptr} + 64], 0xD8",   // Colors block 3
            "vpermq {ymm12}, [{colors_ptr} + 96], 0xD8",  // Colors block 4

            // Load all indices blocks
            "vpermq {ymm1}, [{indices_ptr}], 0xD8",       // Indices block 1
            "vpermq {ymm5}, [{indices_ptr} + 32], 0xD8",  // Indices block 2
            "vpermq {ymm9}, [{indices_ptr} + 64], 0xD8",  // Indices block 3
            "vpermq {ymm13}, [{indices_ptr} + 96], 0xD8", // Indices block 4

            // Process all interleaves - can happen in parallel
            // Block 1
            "vpunpckldq {ymm2}, {ymm0}, {ymm1}",
            "vpunpckhdq {ymm3}, {ymm0}, {ymm1}",

            // Block 2
            "vpunpckldq {ymm6}, {ymm4}, {ymm5}",
            "vpunpckhdq {ymm7}, {ymm4}, {ymm5}",

            // Block 3
            "vpunpckldq {ymm10}, {ymm8}, {ymm9}",
            "vpunpckhdq {ymm11}, {ymm8}, {ymm9}",

            // Block 4
            "vpunpckldq {ymm14}, {ymm12}, {ymm13}",
            "vpunpckhdq {ymm15}, {ymm12}, {ymm13}",

            // Store all results - try to maintain some spacing between writes
            "vmovdqu [{dst_ptr}], {ymm2}",
            "vmovdqu [{dst_ptr} + 32], {ymm3}",
            "vmovdqu [{dst_ptr} + 64], {ymm6}",
            "vmovdqu [{dst_ptr} + 96], {ymm7}",
            "vmovdqu [{dst_ptr} + 128], {ymm10}",
            "vmovdqu [{dst_ptr} + 160], {ymm11}",
            "vmovdqu [{dst_ptr} + 192], {ymm14}",
            "vmovdqu [{dst_ptr} + 224], {ymm15}",

            // Update pointers for next iteration
            "add {colors_ptr}, 128",
            "add {indices_ptr}, 128",
            "add {dst_ptr}, 256",

            // Continue if we haven't reached the end
            "cmp {indices_ptr}, {end}",
            "jb 2b",

            colors_ptr = inout(reg) input_ptr,
            indices_ptr = out(reg) _,
            dst_ptr = inout(reg) output_ptr,
            len_half = in(reg) len / 2,
            end = out(reg) _,
            // Use all 16 YMM registers available in x86-64
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
            ymm12 = out(ymm_reg) _,
            ymm13 = out(ymm_reg) _,
            ymm14 = out(ymm_reg) _,
            ymm15 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 64 (processes 32 bytes of input/output per iteration)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
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
#[target_feature(enable = "avx2")]
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

            // Permute first chunk
            "vperm2f128 {ymm1}, {ymm0}, {ymm2}, 32", // [c0 i0 c1 i1 | c2 i2 c3 i3]
            "vperm2f128 {ymm0}, {ymm0}, {ymm2}, 49", // [c4 i4 c5 i5 | c6 i6 c7 i7]

            // Unpack second 32-byte chunk
            "vunpckhps {ymm5}, {ymm3}, {ymm4}",      // [c10 i10 c11 i11 | c14 i14 c15 i15]
            "vunpcklps {ymm3}, {ymm3}, {ymm4}",      // [c8 i8 c9 i9 | c12 i12 c13 i13]

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
#[target_feature(enable = "avx2")]
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
    #[case::avx_unpack(TestCase {
        name: "avx_unpack",
        func: unpck_detransform,
        min_blocks: 8,  // 64-byte alignment requirement
        many_blocks: 1024,
    })]
    #[case::avx_permd(TestCase {
        name: "avx_permd",
        func: permd_detransform,
        min_blocks: 8,  // 64-byte alignment requirement
        many_blocks: 1024,
    })]
    #[case::avx_unpack_unroll_2(TestCase {
        name: "avx_unpack_unroll_2",
        func: unpck_detransform_unroll_2,
        min_blocks: 16,  // 128-byte alignment requirement
        many_blocks: 1024,
    })]
    #[case::avx_permd_unroll_2(TestCase {
        name: "avx_permd_unroll_2",
        func: permd_detransform_unroll_2,
        min_blocks: 16,  // 128-byte alignment requirement
        many_blocks: 1024,
    })]
    #[cfg_attr(target_arch = "x86_64", case::avx_permd_unroll_4(TestCase {
        name: "avx_permd_unroll_4",
        func: permd_detransform_unroll_4,
        min_blocks: 32,  // 256-byte alignment requirement
        many_blocks: 1024,
    }))]
    #[cfg_attr(target_arch = "x86_64", case::avx_unpack_unroll_4(TestCase {
        name: "avx_unpack_unroll_4",
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

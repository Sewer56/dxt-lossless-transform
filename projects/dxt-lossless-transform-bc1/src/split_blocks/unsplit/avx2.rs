use super::u32_detransform_with_separate_pointers;
use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permd_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
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
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                ymm3 = out(ymm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permd_detransform_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;

    permd_detransform_unroll_2_with_components(output_ptr, len, indices_ptr, colors_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permd_detransform_unroll_2_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_ptr: *const u8,
    mut colors_ptr: *const u8,
) {
    debug_assert!(len % 8 == 0, "len must be divisible by 8");
    let aligned_len = len - (len % 128);
    let colors_aligned_end = colors_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
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
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
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

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            colors_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
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
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn unpck_detransform_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
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
                "vperm2f128 {ymm1}, {ymm0}, {ymm2}, 32",  // [c0 i0 c1 i1 | c2 i2 c3 i3]
                "vperm2f128 {ymm0}, {ymm0}, {ymm2}, 49",  // [c4 i4 c5 i5 | c6 i6 c7 i7]

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

                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
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

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(unpck_detransform, "avx_unpack")]
    #[case(permd_detransform, "avx_permd")]
    #[case(unpck_detransform_unroll_2, "avx_unpack_unroll_2")]
    #[case(permd_detransform_unroll_2, "avx_permd_unroll_2")]
    fn test_avx2_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform using standard implementation
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Reconstruct using the implementation being tested
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(unpck_detransform, "avx_unpack")]
    #[case(permd_detransform, "avx_permd")]
    #[case(unpck_detransform_unroll_2, "avx_unpack_unroll_2")]
    #[case(permd_detransform_unroll_2, "avx_permd_unroll_2")]
    fn test_avx2_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            unsafe {
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);

            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed_unaligned.as_ptr().add(1),
                    reconstructed.as_mut_ptr().add(1),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}

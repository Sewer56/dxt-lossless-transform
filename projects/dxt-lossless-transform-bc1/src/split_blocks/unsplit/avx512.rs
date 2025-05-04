use super::u32_detransform_with_separate_pointers;
use core::arch::asm;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512vl")]
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;

    permute_512_detransform_unroll_2_with_components(output_ptr, len, indices_ptr, colors_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512vl")]
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_detransform_unroll_2_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_ptr: *const u8,
    mut colors_ptr: *const u8,
) {
    debug_assert!(len % 8 == 0, "len must be divisible by 8");
    let aligned_len = len - (len % 256);
    let colors_aligned_end = colors_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        // Define permutation constants for vpermt2d
        // For gathering low dwords (0,16,1,17,etc.)
        const PERM_LOW_BYTES: [i8; 16] = [0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23];
        // For gathering high dwords (8,24,9,25,etc.)
        const PERM_HIGH_BYTES: [i8; 16] =
            [8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31];

        // Load permutation patterns using vpmovsxbd (sign-extend bytes to dwords)
        let perm_low = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_LOW_BYTES.as_ptr() as *const _));
        let perm_high = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_HIGH_BYTES.as_ptr() as *const _));

        unsafe {
            asm!(
                // Align the loop's instruction address to 32 bytes
                ".p2align 5",
                "2:",

                // Load colors and indices
                "vmovdqu64 {zmm4}, [{colors_ptr}]",             // First colors block
                "vmovdqu64 {zmm5}, [{colors_ptr} + 64]",        // Second colors block
                "add {colors_ptr}, 128",
                "vmovdqu64 {zmm3}, [{indices_ptr}]",            // First indices block
                "vmovdqu64 {zmm2}, [{indices_ptr} + 64]",       // Second indices block
                "add {indices_ptr}, 128",

                // Apply permutations
                "vmovdqa64 {zmm6}, {zmm4}",                     // Copy colors for first permutation
                "vpermt2d {zmm6}, {perm_low}, {zmm3}",          // Permute with low pattern - using zmm3 (first indices block)
                "vpermt2d {zmm4}, {perm_high}, {zmm3}",         // Permute with high pattern - using zmm3 (first indices block)

                "vmovdqa64 {zmm3}, {zmm5}",                     // Copy colors for second permutation - reusing zmm3
                "vpermt2d {zmm3}, {perm_low}, {zmm2}",          // Permute with low pattern - using zmm2 (second indices block)
                "vpermt2d {zmm5}, {perm_high}, {zmm2}",         // Permute with high pattern - using zmm2 (second indices block)

                // Store results
                "vmovdqu64 [{dst_ptr}], {zmm6}",                // Store first low part
                "vmovdqu64 [{dst_ptr} + 64], {zmm4}",           // Store first high part
                "vmovdqu64 [{dst_ptr} + 128], {zmm3}",          // Store second low part
                "vmovdqu64 [{dst_ptr} + 192], {zmm5}",          // Store second high part

                // Update pointer and loop.
                "add {dst_ptr}, 256",
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                perm_low = in(zmm_reg) perm_low,
                perm_high = in(zmm_reg) perm_high,
                zmm2 = out(zmm_reg) _,
                zmm3 = out(zmm_reg) _,
                zmm4 = out(zmm_reg) _,
                zmm5 = out(zmm_reg) _,
                zmm6 = out(zmm_reg) _,
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
    #[case(permute_512_detransform_unroll_2, "avx512_permute_unroll_2")]
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
    #[case(permute_512_detransform_unroll_2, "avx512_permute_unroll_2")]
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

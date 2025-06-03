use core::arch::*;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

use crate::transforms::standard::transform::u32_with_separate_pointers;

// Byte indices for vpermt2d to gather color pairs (dword elements 0, 2, 4, ..., 14 from src1 and src2)
const PERM_COLORS_BYTES: [i8; 16] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30];

// Byte indices for vpermt2d to gather index dwords (dword elements 1, 3, 5, ..., 15 from src1 and src2)
const PERM_INDICES_BYTES: [i8; 16] = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let colors_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    permute_512_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let colors_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    permute_512_unroll_2_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
}

/// AVX512 variant that writes colors and indices to separate pointers
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8
/// - The color and index buffers must not overlap with each other or the input buffer
#[allow(unused_assignments)] // no feature for 512
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    let aligned_len = len - (len % 128);

    if aligned_len > 0 {
        let aligned_end_input = input_ptr.add(aligned_len);

        // Load permutation patterns using vpmovsxbd (sign-extend bytes to dwords)
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_INDICES_BYTES.as_ptr() as *const _));

        unsafe {
            asm!(

                // Align the loop's instruction address to 32 bytes.
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX512 will be one of those.
                ".p2align 5",
                "2:",

                // Load all 256 bytes first to utilize memory pipeline
                "vmovdqu64 {zmm2}, [{src_ptr}]",
                "vmovdqu64 {zmm3}, [{src_ptr} + 64]",
                "add {src_ptr}, 128",

                // Do all permutations together to utilize shuffle units (vpermt2d)
                "vmovdqa64 {zmm6}, {zmm2}", // Preserve zmm2 for index permutation
                "vpermt2d {zmm6}, {perm_colors}, {zmm3}", // colors from zmm2/zmm3
                "vpermt2d {zmm2}, {perm_indices}, {zmm3}", // indices from zmm2/zmm3

                // Store all results together to utilize store pipeline
                "vmovdqu64 [{colors_ptr}], {zmm6}",
                "add {colors_ptr}, 64",

                "vmovdqu64 [{indices_ptr}], {zmm2}",
                "add {indices_ptr}, 64",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end_ptr}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end_ptr = in(reg) aligned_end_input,
                perm_colors = in(zmm_reg) perm_colors,
                perm_indices = in(zmm_reg) perm_indices,
                zmm2 = out(zmm_reg) _,
                zmm3 = out(zmm_reg) _,
                zmm6 = out(zmm_reg) _,
                options(nostack, preserves_flags)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, remaining);
    }
}

/// AVX512 variant with 2x unroll that writes colors and indices to separate pointers
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8
/// - The color and index buffers must not overlap with each other or the input buffer
#[allow(unused_assignments)] // no feature for 512
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_unroll_2_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    let aligned_len = len - (len % 256);

    if aligned_len > 0 {
        let aligned_end_input = input_ptr.add(aligned_len);

        // Load permutation patterns using vpmovsxbd (sign-extend bytes to dwords)
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_INDICES_BYTES.as_ptr() as *const _));

        unsafe {
            asm!(

                // Align the loop's instruction address to 32 bytes.
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX512 will be one of those.
                ".p2align 5",
                "2:",

                // Load all 256 bytes first to utilize memory pipeline
                "vmovdqu64 {zmm2}, [{src_ptr}]",
                "vmovdqu64 {zmm3}, [{src_ptr} + 64]",
                "vmovdqu64 {zmm4}, [{src_ptr} + 128]",
                "vmovdqu64 {zmm5}, [{src_ptr} + 192]",
                "add {src_ptr}, 256",

                // Do all permutations together to utilize shuffle units (vpermt2d)
                "vmovdqa64 {zmm6}, {zmm2}", // Preserve zmm2 for index permutation
                "vpermt2d {zmm6}, {perm_colors}, {zmm3}", // colors from zmm2/zmm3
                "vpermt2d {zmm2}, {perm_indices}, {zmm3}", // indices from zmm2/zmm3

                "vmovdqa64 {zmm3}, {zmm4}", // Preserve zmm4 for index permutation, reuse zmm3
                "vpermt2d {zmm3}, {perm_colors}, {zmm5}", // colors from zmm4/zmm5
                "vpermt2d {zmm4}, {perm_indices}, {zmm5}", // indices from zmm4/zmm5

                // Store all results together to utilize store pipeline
                "vmovdqu64 [{colors_ptr}], {zmm6}",
                "vmovdqu64 [{colors_ptr} + 64], {zmm3}",
                "add {colors_ptr}, 128",

                "vmovdqu64 [{indices_ptr}], {zmm2}",
                "vmovdqu64 [{indices_ptr} + 64], {zmm4}",
                "add {indices_ptr}, 128",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end_ptr}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end_ptr = in(reg) aligned_end_input,
                perm_colors = in(zmm_reg) perm_colors,
                perm_indices = in(zmm_reg) perm_indices,
                zmm2 = out(zmm_reg) _,
                zmm3 = out(zmm_reg) _,
                zmm4 = out(zmm_reg) _,
                zmm5 = out(zmm_reg) _,
                zmm6 = out(zmm_reg) _,
                options(nostack, preserves_flags)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, remaining);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::standard::transform::tests::assert_implementation_matches_reference;
    use crate::transforms::standard::transform::tests::generate_bc1_test_data;
    use crate::transforms::standard::transform::tests::transform_with_reference_implementation;
    use core::ptr::copy_nonoverlapping;
    use dxt_lossless_transform_common::allocate::allocate_align_64;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(permute_512, "avx512 permute")]
    #[case(permute_512_unroll_2, "avx512 permute unroll 2")]
    fn test_avx512_aligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let mut input = allocate_align_64(num_blocks * 8).unwrap();
            let mut output_expected = allocate_align_64(input.len()).unwrap();
            let mut output_test = allocate_align_64(input.len()).unwrap();

            unsafe {
                copy_nonoverlapping(
                    generate_bc1_test_data(num_blocks).as_ptr(),
                    input.as_mut_ptr(),
                    input.len(),
                );
            }

            transform_with_reference_implementation(
                input.as_slice(),
                output_expected.as_mut_slice(),
            );

            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                output_test.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(permute_512, "avx512 permute")]
    #[case(permute_512_unroll_2, "avx512 permute unroll 2")]
    fn test_avx512_unaligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);

            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(
                    input_unaligned.as_ptr().add(1),
                    output_test.as_mut_ptr().add(1),
                    input.len(),
                );
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                &output_test[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }

    #[test]
    fn avx512_split_blocks_with_separate_pointers_matches_split_blocks() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut output_ref = allocate_align_64(len).unwrap();
            let mut colors_sep = allocate_align_64(len / 2).unwrap();
            let mut indices_sep = allocate_align_64(len / 2).unwrap();

            unsafe {
                // Reference: AVX512 contiguous output
                permute_512(input.as_ptr(), output_ref.as_mut_ptr(), len);

                // Test: AVX512 separate pointers variant
                permute_512_with_separate_pointers(
                    input.as_ptr(),
                    colors_sep.as_mut_ptr() as *mut u32,
                    indices_sep.as_mut_ptr() as *mut u32,
                    len,
                );
            }

            // Compare colors section (first half)
            assert_eq!(
                &output_ref.as_slice()[0..len / 2],
                colors_sep.as_slice(),
                "AVX512 colors section doesn't match for {num_blocks} blocks"
            );

            // Compare indices section (second half)
            assert_eq!(
                &output_ref.as_slice()[len / 2..],
                indices_sep.as_slice(),
                "AVX512 indices section doesn't match for {num_blocks} blocks"
            );
        }
    }

    #[test]
    fn avx512_unroll_2_split_blocks_with_separate_pointers_matches_split_blocks() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut output_ref = allocate_align_64(len).unwrap();
            let mut colors_sep = allocate_align_64(len / 2).unwrap();
            let mut indices_sep = allocate_align_64(len / 2).unwrap();

            unsafe {
                // Reference: AVX512 unroll 2 contiguous output
                permute_512_unroll_2(input.as_ptr(), output_ref.as_mut_ptr(), len);

                // Test: AVX512 unroll 2 separate pointers variant
                permute_512_unroll_2_with_separate_pointers(
                    input.as_ptr(),
                    colors_sep.as_mut_ptr() as *mut u32,
                    indices_sep.as_mut_ptr() as *mut u32,
                    len,
                );
            }

            // Compare colors section (first half)
            assert_eq!(
                &output_ref.as_slice()[0..len / 2],
                colors_sep.as_slice(),
                "AVX512 unroll 2 colors section doesn't match for {num_blocks} blocks"
            );

            // Compare indices section (second half)
            assert_eq!(
                &output_ref.as_slice()[len / 2..],
                indices_sep.as_slice(),
                "AVX512 unroll 2 indices section doesn't match for {num_blocks} blocks"
            );
        }
    }
}

use crate::split_blocks::split::portable32::u32_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut indices_ptr = output_ptr.add(len / 2);
    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Load permutation indices for vpermd (group colors/indices)
                // Colors go to low lane (0,1,2,3), indices to high lane (4,5,6,7)
                "vmovdqu {ymm0}, [{perm}]",

                // Align the loop's instruction address to 32 bytes for AVX2
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load 64 bytes
                "vmovdqu {ymm1}, [{src_ptr}]",
                "vmovdqu {ymm2}, [{src_ptr} + 32]",
                "add {src_ptr}, 64",  // src += 64

                // Use vpermd to group colors/indices
                "vpermd {ymm3}, {ymm0}, {ymm1}",
                "vpermd {ymm4}, {ymm0}, {ymm2}",

                // Use vperm2i128 to get all colors in one register and all indices in another
                "vperm2i128 {ymm5}, {ymm3}, {ymm4}, 0x20",  // get all colors
                "vperm2i128 {ymm6}, {ymm3}, {ymm4}, 0x31",  // get all indices

                // Store results
                "vmovdqu [{colors_ptr}], {ymm5}",
                "vmovdqu [{indices_ptr}], {ymm6}",

                // Update pointers
                "add {colors_ptr}, 32",
                "add {indices_ptr}, 32",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) output_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
                // TODO: Move this to static.
                // There isn't just a clean way to do it without cloning the whole asm! for x86 (32-bit)
                perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                ymm3 = out(ymm_reg) _,
                ymm4 = out(ymm_reg) _,
                ymm5 = out(ymm_reg) _,
                ymm6 = out(ymm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u32,
            indices_ptr as *mut u32,
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
pub unsafe fn permute_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

    let mut indices_ptr = output_ptr.add(len / 2);
    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Load permutation indices for vpermd (group colors/indices)
                // Colors go to low lane (0,1,2,3), indices to high lane (4,5,6,7)
                "vmovdqu {ymm0}, [{perm}]",

                // Align the loop's instruction address to 32 bytes for AVX2
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load all inputs first to utilize memory pipeline
                "vmovdqu {ymm1}, [{src_ptr}]",
                "vmovdqu {ymm2}, [{src_ptr} + 32]",
                "vmovdqu {ymm3}, [{src_ptr} + 64]",
                "vmovdqu {ymm4}, [{src_ptr} + 96]",
                "add {src_ptr}, 128",  // src += 128

                // Use vpermd to group colors in low lane and indices in high lane
                "vpermd {ymm5}, {ymm0}, {ymm1}",
                "vpermd {ymm6}, {ymm0}, {ymm2}",
                "vpermd {ymm1}, {ymm0}, {ymm3}",  // second block (reuse ymm1/2)
                "vpermd {ymm2}, {ymm0}, {ymm4}",

                // Do all vperm2i128 operations
                "vperm2i128 {ymm3}, {ymm5}, {ymm6}, 0x20",  // all colors
                "vperm2i128 {ymm4}, {ymm5}, {ymm6}, 0x31",  // all indices
                "vperm2i128 {ymm5}, {ymm1}, {ymm2}, 0x20",  // all colors
                "vperm2i128 {ymm6}, {ymm1}, {ymm2}, 0x31",  // all indices

                // Store all results
                "vmovdqu [{colors_ptr}], {ymm3}",      // Store all colors
                "vmovdqu [{colors_ptr} + 32], {ymm5}",
                "add {colors_ptr}, 64",
                "vmovdqu [{indices_ptr}], {ymm4}",      // Store all indices
                "vmovdqu [{indices_ptr} + 32], {ymm6}",
                "add {indices_ptr}, 64",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) output_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
                perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                ymm3 = out(ymm_reg) _,
                ymm4 = out(ymm_reg) _,
                ymm5 = out(ymm_reg) _,
                ymm6 = out(ymm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u32,
            indices_ptr as *mut u32,
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
pub unsafe fn gather(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

    let mut indices_ptr = output_ptr.add(len / 2);
    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Load gather indices into lower ymm registers
                "vmovdqu {ymm0}, [{color_idx}]",  // for colors (0,2,4,6,8,10,12,14)
                "vmovdqu {ymm1}, [{index_idx}]",  // for indices (1,3,5,7,9,11,13,15)

                // Align the loop's instruction address to 32 bytes for AVX2
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Gather colors using even indices
                "vpcmpeqd {ymm2}, {ymm2}, {ymm2}",
                "vpgatherdd {ymm3}, [{src_ptr} + {ymm0} * 4], {ymm2}",  // scale = 4 for 32-bit elements

                // Gather indices using odd indices
                "vpcmpeqd {ymm2}, {ymm2}, {ymm2}",
                "vpgatherdd {ymm4}, [{src_ptr} + {ymm1} * 4], {ymm2}",  // scale = 4 for 32-bit elements
                "add {src_ptr}, 64",   // src += 64

                // Store results
                "vmovdqu [{colors_ptr}], {ymm3}",
                "add {colors_ptr}, 32",
                "vmovdqu [{indices_ptr}], {ymm4}",
                "add {indices_ptr}, 32",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) output_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
                color_idx = in(reg) &[0, 2, 4, 6, 8, 10, 12, 14u32],
                index_idx = in(reg) &[1, 3, 5, 7, 9, 11, 13, 15u32],
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                ymm3 = out(ymm_reg) _,
                ymm4 = out(ymm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u32,
            indices_ptr as *mut u32,
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
pub unsafe fn shuffle_permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut indices_ptr = output_ptr.add(len / 2);
    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Align the loop's instruction address to 32 bytes.
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load 64 bytes
                "vmovdqu {ymm0}, [{src_ptr}]",
                "vmovdqu {ymm1}, [{src_ptr} + 32]",

                // Update source pointer early
                "add {src_ptr}, 64",  // src += 64

                // Do shuffles
                "vshufps {ymm2}, {ymm0}, {ymm1}, 136",  // colors (0b10001000)
                "vpermpd {ymm2}, {ymm2}, 216",  // arrange colors (0b11011000)
                "vshufps {ymm3}, {ymm0}, {ymm1}, 221",  // indices (0b11011101)
                "vpermpd {ymm3}, {ymm3}, 216",  // arrange indices (0b11011000)

                // Store results
                "vmovdqu [{colors_ptr}], {ymm2}",  // Store colors
                "vmovdqu [{indices_ptr}], {ymm3}",      // Store indices
                "add {colors_ptr}, 32",  // colors_ptr += 32
                "add {indices_ptr}, 32",      // indices_ptr += 32

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) output_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
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
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u32,
            indices_ptr as *mut u32,
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
pub unsafe fn shuffle_permute_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let colors_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Call with separate pointers for colors and indices
    shuffle_permute_unroll_2_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8 (BC1 block size)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn shuffle_permute_unroll_2_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Align the loop's instruction address to 32 bytes.
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load all 128 bytes first to utilize memory pipeline
                "vmovdqu {ymm0}, [{src_ptr}]",
                "vmovdqu {ymm1}, [{src_ptr} + 32]",
                "vmovdqu {ymm4}, [{src_ptr} + 64]",
                "vmovdqu {ymm5}, [{src_ptr} + 96]",
                "add {src_ptr}, 128",  // src += 128

                // Do all shuffles together to utilize shuffle units
                "vshufps {ymm2}, {ymm0}, {ymm1}, 136",  // colors (0b10001000)
                "vpermpd {ymm2}, {ymm2}, 216",  // arrange colors (0b11011000)
                "vshufps {ymm3}, {ymm0}, {ymm1}, 221",  // indices (0b11011101)
                "vpermpd {ymm3}, {ymm3}, 216",  // arrange indices (0b11011000)
                "vshufps {ymm6}, {ymm4}, {ymm5}, 136",  // colors (0b10001000)
                "vpermpd {ymm6}, {ymm6}, 216",  // arrange colors (0b11011000)
                "vshufps {ymm7}, {ymm4}, {ymm5}, 221",  // indices (0b11011101)
                "vpermpd {ymm7}, {ymm7}, 216",  // arrange indices (0b11011000)

                // Store all results together to utilize store pipeline
                "vmovdqu [{colors_ptr}], {ymm2}",      // Store colors
                "vmovdqu [{indices_ptr}], {ymm3}",      // Store indices
                "vmovdqu [{colors_ptr} + 32], {ymm6}",  // Store colors
                "vmovdqu [{indices_ptr} + 32], {ymm7}",  // Store indices
                "add {colors_ptr}, 64",   // colors_ptr += 64
                "add {indices_ptr}, 64",   // indices_ptr += 64

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
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
        u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, remaining);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::{
        assert_implementation_matches_reference, generate_bc1_test_data,
        transform_with_reference_implementation,
    };
    use core::ptr::copy_nonoverlapping;
    use dxt_lossless_transform_common::allocate::allocate_align_64;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(shuffle_permute, "shuffle_permute")]
    #[case(shuffle_permute_unroll_2, "shuffle_permute unroll 2")]
    #[case(permute, "permute")]
    #[case(permute_unroll_2, "permute unroll 2")]
    #[case(gather, "gather")]
    fn test_avx2_aligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
            return;
        }

        for num_blocks in 1..=512 {
            let mut input = allocate_align_64(num_blocks * 8).unwrap();
            let mut output_expected = allocate_align_64(input.len()).unwrap();
            let mut output_test = allocate_align_64(input.len()).unwrap();

            // Fill the input with test data
            unsafe {
                copy_nonoverlapping(
                    generate_bc1_test_data(num_blocks).as_ptr(),
                    input.as_mut_ptr(),
                    input.len(),
                );
            }

            // Generate reference output
            transform_with_reference_implementation(
                input.as_slice(),
                output_expected.as_mut_slice(),
            );

            // Test the specific implementation
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
    #[case(shuffle_permute, "shuffle_permute")]
    #[case(shuffle_permute_unroll_2, "shuffle_permute unroll 2")]
    #[case(permute, "permute")]
    #[case(permute_unroll_2, "permute unroll 2")]
    #[case(gather, "gather")]
    fn test_avx2_unaligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            output_test.as_mut_slice().fill(0);
            unsafe {
                // Use pointers offset by 1 byte to create unaligned access
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
    fn avx2_split_blocks_with_separate_pointers_matches_split_blocks() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut output_ref = allocate_align_64(len).unwrap();
            let mut colors_sep = allocate_align_64(len / 2).unwrap();
            let mut indices_sep = allocate_align_64(len / 2).unwrap();

            unsafe {
                // Reference: contiguous output using shuffle_permute_unroll_2
                shuffle_permute_unroll_2(input.as_ptr(), output_ref.as_mut_ptr(), len);

                // Test separate pointers variant
                shuffle_permute_unroll_2_with_separate_pointers(
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
                "AVX2 colors section doesn't match for {num_blocks} blocks"
            );

            // Compare indices section (second half)
            assert_eq!(
                &output_ref.as_slice()[len / 2..],
                indices_sep.as_slice(),
                "AVX2 indices section doesn't match for {num_blocks} blocks"
            );
        }
    }
}

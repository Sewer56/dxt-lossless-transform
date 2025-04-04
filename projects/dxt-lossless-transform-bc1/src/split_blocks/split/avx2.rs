use crate::split_blocks::split::portable32::u32_with_separate_pointers;
use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
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
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn permute_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
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
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn permute_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 256-byte blocks as possible
    let aligned_len = len - (len % 256);

    let mut indices_ptr = output_ptr.add(len / 2);
    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Load permutation indices for vpermd
                "vmovdqu ymm15, [{perm}]",

                // Align the loop's instruction address to 32 bytes for AVX2
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load 256 bytes (32 blocks)
                "vmovdqu ymm0, [{src_ptr}]",
                "vmovdqu ymm1, [{src_ptr} + 32]",
                "vmovdqu ymm4, [{src_ptr} + 64]",
                "vmovdqu ymm5, [{src_ptr} + 96]",
                "vmovdqu ymm8, [{src_ptr} + 128]",
                "vmovdqu ymm9, [{src_ptr} + 160]",
                "vmovdqu ymm12, [{src_ptr} + 192]",
                "vmovdqu ymm13, [{src_ptr} + 224]",
                "add {src_ptr}, 256",  // src += 256

                // Use vpermd to group colors in low lane and indices in high lane
                "vpermd ymm2, ymm15, ymm0",
                "vpermd ymm3, ymm15, ymm1",
                "vpermd ymm6, ymm15, ymm4",
                "vpermd ymm7, ymm15, ymm5",
                "vpermd ymm10, ymm15, ymm8",
                "vpermd ymm11, ymm15, ymm9",
                "vpermd ymm14, ymm15, ymm12",
                "vpermd ymm0, ymm15, ymm13",

                // Do all vperm2i128 operations
                "vperm2i128 ymm1, ymm2, ymm3, 0x20", // all colors
                "vperm2i128 ymm2, ymm2, ymm3, 0x31", // all indices
                "vperm2i128 ymm3, ymm6, ymm7, 0x20", // all colors
                "vperm2i128 ymm4, ymm6, ymm7, 0x31", // all indices
                "vperm2i128 ymm5, ymm10, ymm11, 0x20", // all colors
                "vperm2i128 ymm6, ymm10, ymm11, 0x31", // all indices
                "vperm2i128 ymm7, ymm14, ymm0, 0x20", // all colors
                "vperm2i128 ymm8, ymm14, ymm0, 0x31", // all indices

                // Store all results
                "vmovdqu [{colors_ptr}], ymm1",
                "vmovdqu [{colors_ptr} + 32], ymm3",
                "vmovdqu [{colors_ptr} + 64], ymm5",
                "vmovdqu [{colors_ptr} + 96], ymm7",
                "add {colors_ptr}, 128",

                "vmovdqu [{indices_ptr}], ymm2",
                "vmovdqu [{indices_ptr} + 32], ymm4",
                "vmovdqu [{indices_ptr} + 64], ymm6",
                "vmovdqu [{indices_ptr} + 96], ymm8",
                "add {indices_ptr}, 128",

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) output_ptr,
                indices_ptr = inout(reg) indices_ptr,
                end = inout(reg) aligned_end,
                perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
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
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn gather(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
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
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn shuffle_permute(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
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
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub unsafe fn shuffle_permute_unroll_2(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

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
                colors_ptr = inout(reg) output_ptr,
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
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u32,
            indices_ptr as *mut u32,
            remaining,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::tests::transform_with_reference_implementation;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(32)]
    #[case(33)]
    #[case(513)]
    #[case(2048)]
    fn test_avx2_implementations(#[case] num_blocks: usize) {
        let input = generate_bc1_test_data(num_blocks);
        let mut output_expected = allocate_align_64(input.len());
        let mut output_test = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), output_expected.as_mut_slice());

        // Test each implementation
        let implementations = [
            (shuffle_permute as PermuteFn, "shuffle_permute"),
            (
                shuffle_permute_unroll_2 as PermuteFn,
                "shuffle_permute unroll 2",
            ),
            (permute as PermuteFn, "permute"),
            (permute_unroll_2 as PermuteFn, "permute unroll 2"),
            (permute_unroll_4 as PermuteFn, "permute unroll 4"),
            (gather as PermuteFn, "gather"),
        ];

        for (permute_fn, impl_name) in implementations {
            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }
            assert_eq!(
                output_expected.as_slice(),
                output_test.as_slice(),
                "{} implementation produced different results than reference for {} blocks.\n\
                First differing block will have predictable values:\n\
                Colors: Sequential 1-4 + (block_num * 4)\n\
                Indices: Sequential 128-131 + (block_num * 4)",
                impl_name,
                num_blocks
            );
        }
    }
}

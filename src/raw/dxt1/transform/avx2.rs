use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for SSE operations
#[inline(always)]
pub unsafe fn avx2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    avx2_permute(input_ptr, output_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (for two AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[inline(never)]
pub unsafe fn avx2_permute(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        asm!(
            // Preserve non-volatile registers we'll use
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            // Load permutation indices for vpermd (group colors/indices)
            // Colors go to low lane (0,1,2,3), indices to high lane (4,5,6,7)
            "vmovdqu ymm15, [{perm}]",

            // Calculate end address
            "mov rbx, {src}",
            "add rbx, {len}",  // end = src + len

            // Store pointers in preserved registers
            "mov r12, {src}",     // src
            "mov r13, {dst}",     // dst for colors
            "mov r14, {dst}",     // dst for indices
            "add r14, {len_half}", // indices start halfway through output

            // Align the loop's instruction address to 32 bytes for AVX2
            // This isn't strictly needed, but more modern processors fetch + execute instructions in
            // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
            // assume a processor with AVX2 will be one of those.
            ".p2align 5",
            "2:",  // Local label for loop

            // Load 64 bytes (8 blocks) into YMM0 and YMM1
            "vmovdqa ymm0, [r12]",
            "vmovdqa ymm1, [r12 + 32]",
            "add r12, 64", // src += 64 (four 16-byte blocks)

            // Use vpermd to group colors in low lane and indices in high lane
            "vpermd ymm2, ymm15, ymm0",   // group colors/indices in first register
            "vpermd ymm3, ymm15, ymm1",   // group colors/indices in second register

            // Use vperm2i128 to get all colors in one register and all indices in another
            "vperm2i128 ymm0, ymm2, ymm3, 0x20",  // get all colors
            "vperm2i128 ymm1, ymm2, ymm3, 0x31",  // get all indices

            // Store the results
            "vmovdqa [r13], ymm0",      // Store colors
            "vmovdqa [r14], ymm1",      // Store indices

            // Update pointers
            "add r13, 32",   // colors_ptr += 32 (half of the processed data)
            "add r14, 32",   // indices_ptr += 32 (half of the processed data)

            // Compare against end address and loop if not done
            "cmp r12, rbx",
            "jb 2b",

            // Restore preserved registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            // Clear YMM registers to avoid performance penalties
            "vzeroupper",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            // Load address of permutation indices
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for four AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[inline(never)]
pub unsafe fn avx2_permute_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            "vmovdqu ymm15, [{perm}]",

            "mov rbx, {src}",
            "add rbx, {len}",

            "mov r12, {src}",
            "mov r13, {dst}",
            "mov r14, {dst}",
            "add r14, {len_half}",

            ".p2align 5",
            "2:",

            // Load all inputs first
            "vmovdqa ymm0, [r12]",
            "vmovdqa ymm1, [r12 + 32]",
            "vmovdqa ymm4, [r12 + 64]",
            "vmovdqa ymm5, [r12 + 96]",
            "add r12, 128",

            // Do all vpermd operations
            "vpermd ymm2, ymm15, ymm0",
            "vpermd ymm3, ymm15, ymm1",
            "vpermd ymm6, ymm15, ymm4",
            "vpermd ymm7, ymm15, ymm5",

            // Do all vperm2i128 operations
            "vperm2i128 ymm0, ymm2, ymm3, 0x20",
            "vperm2i128 ymm1, ymm2, ymm3, 0x31",
            "vperm2i128 ymm4, ymm6, ymm7, 0x20",
            "vperm2i128 ymm5, ymm6, ymm7, 0x31",

            // Store all results
            "vmovdqa [r13], ymm0",
            "vmovdqa [r14], ymm1",
            "vmovdqa [r13 + 32], ymm4",
            "vmovdqa [r14 + 32], ymm5",

            "add r13, 64",
            "add r14, 64",

            "cmp r12, rbx",
            "jb 2b",

            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            "vzeroupper",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 256 (for eight AVX2 registers worth of data)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[inline(never)]
pub unsafe fn avx2_permute_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 256 == 0);

    unsafe {
        asm!(
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            "vmovdqu ymm15, [{perm}]",

            "mov rbx, {src}",
            "add rbx, {len}",

            "mov r12, {src}",
            "mov r13, {dst}",
            "mov r14, {dst}",
            "add r14, {len_half}",

            ".p2align 5",
            "2:",

            // Load all inputs
            "vmovdqa ymm0, [r12]",
            "vmovdqa ymm1, [r12 + 32]",
            "vmovdqa ymm2, [r12 + 64]",
            "vmovdqa ymm3, [r12 + 96]",
            "vmovdqa ymm4, [r12 + 128]",
            "vmovdqa ymm5, [r12 + 160]",
            "vmovdqa ymm6, [r12 + 192]",
            "vmovdqa ymm7, [r12 + 224]",
            "add r12, 256",

            // Do all vpermd operations
            "vpermd ymm8, ymm15, ymm0",
            "vpermd ymm9, ymm15, ymm1",
            "vpermd ymm10, ymm15, ymm2",
            "vpermd ymm11, ymm15, ymm3",
            "vpermd ymm12, ymm15, ymm4",
            "vpermd ymm13, ymm15, ymm5",
            "vpermd ymm14, ymm15, ymm6",
            "vpermd ymm0, ymm15, ymm7",

            // Do all vperm2i128 operations
            "vperm2i128 ymm1, ymm8, ymm9, 0x20",
            "vperm2i128 ymm2, ymm8, ymm9, 0x31",
            "vperm2i128 ymm3, ymm10, ymm11, 0x20",
            "vperm2i128 ymm4, ymm10, ymm11, 0x31",
            "vperm2i128 ymm5, ymm12, ymm13, 0x20",
            "vperm2i128 ymm6, ymm12, ymm13, 0x31",
            "vperm2i128 ymm7, ymm14, ymm0, 0x20",
            "vperm2i128 ymm8, ymm14, ymm0, 0x31",

            // Store all results
            "vmovdqa [r13], ymm1",
            "vmovdqa [r14], ymm2",
            "vmovdqa [r13 + 32], ymm3",
            "vmovdqa [r14 + 32], ymm4",
            "vmovdqa [r13 + 64], ymm5",
            "vmovdqa [r14 + 64], ymm6",
            "vmovdqa [r13 + 96], ymm7",
            "vmovdqa [r14 + 96], ymm8",

            "add r13, 128",
            "add r14, 128",

            "cmp r12, rbx",
            "jb 2b",

            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            "vzeroupper",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            perm = in(reg) &[0, 2, 4, 6, 1, 3, 5, 7u32],
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::dxt1::testutils::allocate_align_64;
    use crate::raw::transform::tests::*;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    fn get_implementations() -> [(PermuteFn, &'static str); 3] {
        [
            (avx2_permute as PermuteFn, "basic"),
            (avx2_permute_unroll_2 as PermuteFn, "unroll 2"),
            (avx2_permute_unroll_4 as PermuteFn, "unroll 4"),
        ]
    }

    #[rstest]
    #[case::min_size(32)] // 256 bytes - minimum size for unroll-4
    #[case::many_unrolls(512)] // 4KB - tests multiple unroll iterations
    #[case::large(2048)] // 16KB - large dataset
    fn test_avx2_implementations(#[case] num_blocks: usize) {
        let input = generate_dxt1_test_data(num_blocks);
        let mut output_expected = allocate_align_64(input.len());
        let mut output_test = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), output_expected.as_mut_slice());

        // Test each implementation
        for (permute_fn, impl_name) in get_implementations() {
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

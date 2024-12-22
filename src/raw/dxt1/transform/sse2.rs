use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for SSE operations
#[inline(always)]
pub unsafe fn sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    punpckhqdq_unroll_4(input_ptr, output_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len is at least divisible by 128
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn punpckhqdq_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        asm!(
            // Preserve non-volatile registers we'll use
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            // Calculate end address
            "mov rbx, {src}",
            "add rbx, {len}",  // end = src + len

            // Store pointers in preserved registers
            "mov r12, {src}",     // src
            "mov r13, {dst}",     // dst for colors
            "mov r14, {dst}",     // dst for indices
            "add r14, {len_half}", // indices start halfway through output

            // Align the loop's instruction address to 16 bytes
            ".p2align 4",
            "2:",  // Local label for loop

            // Load 16 blocks (128 bytes)
            "movdqa xmm0, [r12]",
            "movdqa xmm1, [r12 + 16]",
            "movdqa xmm2, [r12 + 32]",
            "movdqa xmm3, [r12 + 48]",
            "movdqa xmm4, [r12 + 64]",
            "movdqa xmm5, [r12 + 80]",
            "movdqa xmm6, [r12 + 96]",
            "movdqa xmm7, [r12 + 112]",
            "add r12, 128",  // src += 8 * 16

            // Shuffle all to separate colors and indices
            "pshufd xmm0, xmm0, 0xD8",
            "pshufd xmm1, xmm1, 0xD8",
            "pshufd xmm2, xmm2, 0xD8",
            "pshufd xmm3, xmm3, 0xD8",
            "pshufd xmm4, xmm4, 0xD8",
            "pshufd xmm5, xmm5, 0xD8",
            "pshufd xmm6, xmm6, 0xD8",
            "pshufd xmm7, xmm7, 0xD8",

            // Copy registers for reorganization
            "movdqa xmm8, xmm0",
            "movdqa xmm9, xmm2",
            "movdqa xmm10, xmm4",
            "movdqa xmm11, xmm6",

            // Reorganize all pairs into colors/indices
            "punpckhqdq xmm0, xmm1",     // indices 0,1
            "punpckhqdq xmm2, xmm3",     // indices 2,3
            "punpckhqdq xmm4, xmm5",     // indices 4,5
            "punpckhqdq xmm6, xmm7",     // indices 6,7
            "punpcklqdq xmm8, xmm1",     // colors 0,1
            "punpcklqdq xmm9, xmm3",     // colors 2,3
            "punpcklqdq xmm10, xmm5",    // colors 4,5
            "punpcklqdq xmm11, xmm7",    // colors 6,7

            // Store colors
            "movdqa [r13],      xmm8",
            "movdqa [r13 + 16], xmm9",
            "movdqa [r13 + 32], xmm10",
            "movdqa [r13 + 48], xmm11",

            // Store indices
            "movdqa [r14],      xmm0",
            "movdqa [r14 + 16], xmm2",
            "movdqa [r14 + 32], xmm4",
            "movdqa [r14 + 48], xmm6",

            // Update pointers
            "add r13, 64",   // colors_ptr += 8 * 8
            "add r14, 64",   // indices_ptr += 8 * 8

            // Compare against end address and loop if not done
            "cmp r12, rbx",
            "jb 2b",

            // Restore preserved registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len is at least divisible by 64
/// - pointers must be properly aligned for SSE operations
#[inline(never)]
pub unsafe fn punpckhqdq_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        asm!(
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            "mov rbx, {src}",
            "add rbx, {len}",

            "mov r12, {src}",
            "mov r13, {dst}",
            "mov r14, {dst}",
            "add r14, {len_half}",

            ".p2align 4",
            "2:",

            // Load 8 blocks (64 bytes)
            "movdqa xmm0, [r12]",
            "movdqa xmm1, [r12 + 16]",
            "movdqa xmm2, [r12 + 32]",
            "movdqa xmm3, [r12 + 48]",
            "add r12, 64",   // src += 4 * 16

            // Shuffle all
            "pshufd xmm0, xmm0, 0xD8",
            "pshufd xmm1, xmm1, 0xD8",
            "pshufd xmm2, xmm2, 0xD8",
            "pshufd xmm3, xmm3, 0xD8",

            // Copy registers
            "movdqa xmm4, xmm0",
            "movdqa xmm5, xmm2",

            // Reorganize pairs
            "punpckhqdq xmm0, xmm1",     // indices 0,1
            "punpckhqdq xmm2, xmm3",     // indices 2,3
            "punpcklqdq xmm4, xmm1",     // colors 0,1
            "punpcklqdq xmm5, xmm3",     // colors 2,3

            // Store colors and indices
            "movdqa [r13],      xmm4",
            "movdqa [r13 + 16], xmm5",
            "movdqa [r14],      xmm0",
            "movdqa [r14 + 16], xmm2",

            // Update pointers
            "add r13, 32",   // colors_ptr += 4 * 8
            "add r14, 32",   // indices_ptr += 4 * 8

            "cmp r12, rbx",
            "jb 2b",

            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len is at least divisible by 32
/// - pointers must be properly aligned for SSE operations
#[inline(never)]
pub unsafe fn punpckhqdq_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        asm!(
            "push rbx",
            "push r12",
            "push r13",
            "push r14",

            "mov rbx, {src}",
            "add rbx, {len}",

            "mov r12, {src}",
            "mov r13, {dst}",
            "mov r14, {dst}",
            "add r14, {len_half}",

            ".p2align 4",
            "2:",

            // Load 4 blocks (32 bytes)
            "movdqa xmm0, [r12]",
            "movdqa xmm1, [r12 + 16]",
            "add r12, 32",   // src += 2 * 16

            // Shuffle both
            "pshufd xmm0, xmm0, 0xD8",
            "pshufd xmm1, xmm1, 0xD8",

            // Copy first register
            "movdqa xmm2, xmm0",

            // Reorganize pair
            "punpcklqdq xmm2, xmm1",     // colors
            "punpckhqdq xmm0, xmm1",     // indices

            // Store colors and indices
            "movdqa [r13], xmm2",
            "movdqa [r14], xmm0",

            // Update pointers
            "add r13, 16",   // colors_ptr += 2 * 8
            "add r14, 16",   // indices_ptr += 2 * 8

            "cmp r12, rbx",
            "jb 2b",

            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            src = in(reg) input_ptr,
            dst = in(reg) output_ptr,
            len = in(reg) len,
            len_half = in(reg) len / 2,
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

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::min_size(16)] // 128 bytes - minimum size for unroll-8
    #[case::one_unroll(32)] // 256 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_sse2_implementations(#[case] num_blocks: usize) {
        let input = generate_dxt1_test_data(num_blocks);
        let mut output_expected = allocate_align_64(input.len());
        let mut output_test = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), output_expected.as_mut_slice());

        // Test each SSE2 implementation variant
        let implementations = get_implementations();

        for (impl_name, implementation) in implementations {
            // Clear the output buffer
            output_test.as_mut_slice().fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
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

    #[cfg(target_arch = "x86_64")]
    pub fn get_implementations<'a>() -> [(&'a str, TransformFn); 4] {
        [
            ("SSE2 (auto-selected)", sse2),
            ("SSE2 punpckhqdq unroll-8", punpckhqdq_unroll_8),
            ("SSE2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
            ("SSE2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
        ]
    }

    #[cfg(not(target_arch = "x86_64"))]
    pub fn get_implementations<'a>() -> [(&'a str, TransformFn); 3] {
        [
            ("SSE2 (auto-selected)", sse2),
            ("SSE2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
            ("SSE2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
        ]
    }
}

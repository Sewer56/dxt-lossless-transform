use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
/// - len is at least divisible by 128
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
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
            "movdqu xmm0, [r12]",
            "movdqu xmm1, [r12 + 16]",
            "movdqu xmm2, [r12 + 32]",
            "movdqu xmm3, [r12 + 48]",
            "movdqu xmm4, [r12 + 64]",
            "movdqu xmm5, [r12 + 80]",
            "movdqu xmm6, [r12 + 96]",
            "movdqu xmm7, [r12 + 112]",
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
            "movdqu xmm8, xmm0",
            "movdqu xmm9, xmm2",
            "movdqu xmm10, xmm4",
            "movdqu xmm11, xmm6",

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
            "movdqu [r13],      xmm8",
            "movdqu [r13 + 16], xmm9",
            "movdqu [r13 + 32], xmm10",
            "movdqu [r13 + 48], xmm11",
            "add r13, 64",   // colors_ptr += 8 * 8

            // Store indices
            "movdqu [r14],      xmm0",
            "movdqu [r14 + 16], xmm2",
            "movdqu [r14 + 32], xmm4",
            "movdqu [r14 + 48], xmm6",
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
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len is at least divisible by 64
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn punpckhqdq_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut indices_ptr = output_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
            // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
            ".p2align 5",
            "2:",

            // Load 8 blocks (64 bytes)
            "movdqu {xmm0}, [{src_ptr}]",
            "movdqu {xmm1}, [{src_ptr} + 16]",
            "movdqu {xmm2}, [{src_ptr} + 32]",
            "movdqu {xmm3}, [{src_ptr} + 48]",
            "add {src_ptr}, 64",   // src += 4 * 16

            // Shuffle all
            "pshufd {xmm0}, {xmm0}, 0xD8",
            "pshufd {xmm1}, {xmm1}, 0xD8",
            "pshufd {xmm2}, {xmm2}, 0xD8",
            "pshufd {xmm3}, {xmm3}, 0xD8",

            // Copy registers
            "movdqu {xmm4}, {xmm0}",
            "movdqu {xmm5}, {xmm2}",

            // Reorganize pairs
            "punpckhqdq {xmm0}, {xmm1}",     // indices 0,1
            "punpckhqdq {xmm2}, {xmm3}",     // indices 2,3
            "punpcklqdq {xmm4}, {xmm1}",     // colors 0,1
            "punpcklqdq {xmm5}, {xmm3}",     // colors 2,3

            // Store colors and indices
            "movdqu [{colors_ptr}],      {xmm4}",
            "movdqu [{colors_ptr} + 16], {xmm5}",
            "add {colors_ptr}, 32",   // colors_ptr += 4 * 8
            "movdqu [{indices_ptr}],      {xmm0}",
            "movdqu [{indices_ptr} + 16], {xmm2}",
            "add {indices_ptr}, 32",   // indices_ptr += 4 * 8

            "cmp {src_ptr}, {end}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
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
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn punpckhqdq_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        let mut indices_ptr = output_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
            // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
            ".p2align 5",
            "2:",

            // Load 4 blocks (32 bytes)
            "movdqu {xmm0}, [{src_ptr}]",
            "movdqu {xmm1}, [{src_ptr} + 16]",
            "add {src_ptr}, 32",   // src += 2 * 16

            // Shuffle both
            "pshufd {xmm0}, {xmm0}, 0xD8",
            "pshufd {xmm1}, {xmm1}, 0xD8",

            // Copy first register
            "movdqu {xmm2}, {xmm0}",

            // Reorganize pair
            "punpcklqdq {xmm2}, {xmm1}",     // colors
            "punpckhqdq {xmm0}, {xmm1}",     // indices

            // Store colors and indices
            "movdqu [{colors_ptr}], {xmm2}",
            "add {colors_ptr}, 16",   // colors_ptr += 2 * 8
            "movdqu [{indices_ptr}], {xmm0}",
            "add {indices_ptr}, 16",   // indices_ptr += 2 * 8

            "cmp {src_ptr}, {end}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
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
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn shufps_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        let mut indices_ptr = output_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
            // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
            ".p2align 5",
            "2:",

            // Load 2 blocks (32 bytes)
            "movdqu {xmm0}, [{src_ptr}]",
            "movdqu {xmm1}, [{src_ptr} + 16]",
            "add {src_ptr}, 32",   // src += 2 * 16

            // Shuffle to separate colors and indices
            "movaps {xmm2}, {xmm0}",
            "shufps {xmm2}, {xmm1}, 0x88",    // Colors (0b10001000)
            "shufps {xmm0}, {xmm1}, 0xDD",    // Indices (0b11011101)

            // Store colors and indices
            "movdqu [{colors_ptr}], {xmm2}",
            "add {colors_ptr}, 16",   // colors_ptr += 2 * 8
            "movdqu [{indices_ptr}], {xmm0}",
            "add {indices_ptr}, 16",   // indices_ptr += 2 * 8

            "cmp {src_ptr}, {end}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
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
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn shufps_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut indices_ptr = output_ptr.add(len / 2);
        let mut end = input_ptr.add(len);
        asm!(
            // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
            // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
            ".p2align 5",
            "2:",

            // Load 4 blocks (64 bytes)
            "movdqu {xmm0}, [{src_ptr}]",
            "movdqu {xmm1}, [{src_ptr} + 16]",
            "movdqu {xmm2}, [{src_ptr} + 32]",
            "movdqu {xmm3}, [{src_ptr} + 48]",
            "add {src_ptr}, 64",   // src += 4 * 16

            "movaps {xmm4}, {xmm0}",
            "movaps {xmm5}, {xmm2}",

            // Shuffle the pairs to rearrange
            "shufps {xmm0}, {xmm1}, 0xDD",    // Indices (0b11011101)
            "shufps {xmm2}, {xmm3}, 0xDD",    // Indices (0b11011101)
            "shufps {xmm4}, {xmm1}, 0x88",    // Colors (0b10001000)
            "shufps {xmm5}, {xmm3}, 0x88",    // Colors (0b10001000)

            // Store colors and indices
            "movdqu [{indices_ptr}], {xmm0}",
            "movdqu [{indices_ptr} + 16], {xmm2}",
            "add {indices_ptr}, 32",   // indices_ptr += 4 * 8
            "movdqu [{colors_ptr}], {xmm4}",
            "movdqu [{colors_ptr} + 16], {xmm5}",
            "add {colors_ptr}, 32",   // colors_ptr += 4 * 8

            "cmp {src_ptr}, {end}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            colors_ptr = inout(reg) output_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len is at least divisible by 128
/// - pointers must be properly aligned for SSE operations
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn shufps_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

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

            // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
            // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
            ".p2align 5",
            "2:",

            // Load first 4 blocks (64 bytes)
            "movdqu xmm0, [r12]",
            "movdqu xmm1, [r12 + 16]",
            "movdqu xmm2, [r12 + 32]",
            "movdqu xmm3, [r12 + 48]",

            // Load second 4 blocks (64 bytes)
            "movdqu xmm4, [r12 + 64]",
            "movdqu xmm5, [r12 + 80]",
            "movdqu xmm6, [r12 + 96]",
            "movdqu xmm7, [r12 + 112]",
            "add r12, 128",  // src += 8 * 16

            // First pair shuffle
            "movaps xmm8, xmm0",
            "shufps xmm8, xmm1, 0x88",    // Colors (0b10001000)
            "shufps xmm0, xmm1, 0xDD",    // Indices (0b11011101)

            // Second pair shuffle
            "movaps xmm9, xmm2",
            "shufps xmm9, xmm3, 0x88",    // Colors (0b10001000)
            "shufps xmm2, xmm3, 0xDD",    // Indices (0b11011101)

            // Third pair shuffle
            "movaps xmm10, xmm4",
            "shufps xmm10, xmm5, 0x88",   // Colors (0b10001000)
            "shufps xmm4, xmm5, 0xDD",    // Indices (0b11011101)

            // Fourth pair shuffle
            "movaps xmm11, xmm6",
            "shufps xmm11, xmm7, 0x88",   // Colors (0b10001000)
            "shufps xmm6, xmm7, 0xDD",    // Indices (0b11011101)

            // Store colors
            "movdqu [r13], xmm8",
            "movdqu [r13 + 16], xmm9",
            "movdqu [r13 + 32], xmm10",
            "movdqu [r13 + 48], xmm11",
            "add r13, 64",   // colors_ptr += 8 * 8

            // Store indices
            "movdqu [r14], xmm0",
            "movdqu [r14 + 16], xmm2",
            "movdqu [r14 + 32], xmm4",
            "movdqu [r14 + 48], xmm6",
            "add r14, 64",   // indices_ptr += 8 * 8

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

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::min_size(16)] // 128 bytes - minimum size for unroll-8
    #[case::one_unroll(32)] // 256 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_sse2_implementations(#[case] num_blocks: usize) {
        let input = generate_bc1_test_data(num_blocks);
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
    pub fn get_implementations<'a>() -> [(&'a str, TransformFn); 6] {
        [
            ("SSE2 punpckhqdq unroll-8", punpckhqdq_unroll_8),
            ("SSE2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
            ("SSE2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
            ("SSE2 shuffle unroll-2", shufps_unroll_2),
            ("SSE2 shuffle unroll-4", shufps_unroll_4),
            ("SSE2 shuffle unroll-8", shufps_unroll_8),
        ]
    }

    #[cfg(target_arch = "x86")]
    pub fn get_implementations<'a>() -> [(&'a str, TransformFn); 4] {
        [
            ("SSE2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
            ("SSE2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
            ("SSE2 shuffle unroll-2", shufps_unroll_2),
            ("SSE2 shuffle unroll-4", shufps_unroll_4),
        ]
    }
}

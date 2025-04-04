use crate::split_blocks::split::portable32::u32_with_separate_pointers;
use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn punpckhqdq_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut indices_ptr = output_ptr.add(len / 2);
    let mut aligned_end = input_ptr.add(aligned_len);
    if aligned_len > 0 {
        unsafe {
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
                end = inout(reg) aligned_end,
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
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn punpckhqdq_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 32-byte blocks as possible
    let aligned_len = len - (len % 32);

    let mut indices_ptr = output_ptr.add(len / 2);
    let mut aligned_end = input_ptr.add(aligned_len);
    if aligned_len > 0 {
        unsafe {
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
                end = inout(reg) aligned_end,
                xmm0 = out(xmm_reg) _,
                xmm1 = out(xmm_reg) _,
                xmm2 = out(xmm_reg) _,
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
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn shufps_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 32-byte blocks as possible
    let aligned_len = len - (len % 32);

    let mut indices_ptr = output_ptr.add(len / 2);
    let mut aligned_end = input_ptr.add(aligned_len);
    if aligned_len > 0 {
        unsafe {
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
                end = inout(reg) aligned_end,
                xmm0 = out(xmm_reg) _,
                xmm1 = out(xmm_reg) _,
                xmm2 = out(xmm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr.add(aligned_len),
            output_ptr.add(aligned_len) as *mut u32,
            indices_ptr as *mut u32,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE operations
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn shufps_unroll_4(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut indices_ptr = output_ptr.add(len / 2);
    let mut aligned_end = input_ptr.add(aligned_len);
    if aligned_len > 0 {
        unsafe {
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
                end = inout(reg) aligned_end,
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

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(1)]
    #[case(2)]
    #[case(4)]
    #[case(8)]
    #[case(16)]
    #[case(32)]
    #[case(64)]
    #[case(128)]
    fn test_sse2_implementations(#[case] num_blocks: usize) {
        let input = generate_bc1_test_data(num_blocks);
        let mut output_expected = allocate_align_64(input.len());
        let mut output_test = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), output_expected.as_mut_slice());

        // Test each SSE2 implementation variant
        let implementations: [(&str, TransformFn); 4] = [
            ("SSE2 punpckhqdq unroll-4", punpckhqdq_unroll_4),
            ("SSE2 punpckhqdq unroll-2", punpckhqdq_unroll_2),
            ("SSE2 shuffle unroll-2", shufps_unroll_2),
            ("SSE2 shuffle unroll-4", shufps_unroll_4),
        ];

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
}

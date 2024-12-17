/*
 * DXT1 Block Rearrangement Optimization Explanation
 * =================================================
 *
 * Original sequential DXT1 data layout:
 * Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:
 *
 * Address: 0       4       8   8      12      16
 *          +-------+-------+   +-------+-------+
 * Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
 *          +-------+-------+   +-------+-------+
 *
 * Each 8-byte block contains:
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed indices (sixteen 2-bit indices)
 *
 * Optimized layout separates colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Color endpoints tend to be spatially coherent
 * 2. Index patterns often repeat across blocks
 * 3. Separating them allows better compression of each stream
 *
 * Requirements
 * ============
 *
 * A second, separate buffer to receive the results.
 *
 * While doing it in-place is technically possible, and would be beneficial in the sense that there
 * would be improved cache locality; unfortunately, that is not possible to do in a 'single pass'
 * while maintaining the spatial coherency/order.
 *
 * Introducing a second pass meanwhile would be a performance hit.
 *
 * This is possible to do with either allocating half of a buffer, and then copying the other half back,
 * or outputting it all to a single buffer. Outputting all to single buffer is faster.
 */

use std::arch::asm;

/// Transform into separated color/index format preserving byte order
///
/// # Safety
///
/// The buffer at 'input' and 'output' should be divisible by 8 and equal in length.
#[inline(always)]
pub unsafe fn transform_64bit_portable(input: &[u8], output: &mut [u8]) {
    debug_assert!(input.len() % 8 == 0);
    debug_assert!(output.len() % 8 == 0);
    debug_assert!(input.len() == output.len());

    unsafe {
        let max_ptr = input.as_ptr().add(input.len()) as *mut u64;
        let mut input_ptr = input.as_ptr() as *mut u64;

        // Split output into color and index sections
        let mut colours_ptr = output.as_mut_ptr() as *mut u32;
        let mut indices_ptr = output.as_mut_ptr().add(input.len() / 2) as *mut u32;

        while input_ptr < max_ptr {
            let curr = *input_ptr;

            // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
            let color_value = curr as u32;
            let index_value = (curr >> 32) as u32;

            // Store colours and indices to their respective halves
            *colours_ptr = color_value;
            *indices_ptr = index_value;

            input_ptr = input_ptr.add(1);
            colours_ptr = colours_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
        }
    }
}

/// # Safety
///
/// The buffer at 'input' and 'output' should be divisible by 8 and equal in length.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
pub unsafe fn transform_64bit_sse2(input: &[u8], output: &mut [u8]) {
    debug_assert!(input.len() % 8 == 0);
    debug_assert!(output.len() % 8 == 0);
    debug_assert!(input.len() == output.len());

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
            "mov r13, {dst}",     // colors (first half of output)
            "mov r14, {dst}",     // indices (second half of output)
            "add r14, {len_half}", // indices start halfway through output

            // Align the loop's instruction address to 16 bytes
            ".p2align 4",
            "2:",  // Local label for loop
            // Load 16 blocks (128 bytes)
            "movupd xmm0,  [r12]",
            "movupd xmm1,  [r12 + 16]",
            "movupd xmm2,  [r12 + 32]",
            "movupd xmm3,  [r12 + 48]",
            "movupd xmm4,  [r12 + 64]",
            "movupd xmm5,  [r12 + 80]",
            "movupd xmm6,  [r12 + 96]",
            "movupd xmm7,  [r12 + 112]",
            "movupd xmm8,  [r12 + 128]",
            "movupd xmm9,  [r12 + 144]",
            "movupd xmm10, [r12 + 160]",
            "movupd xmm11, [r12 + 176]",
            "movupd xmm12, [r12 + 192]",
            "movupd xmm13, [r12 + 208]",
            "movupd xmm14, [r12 + 224]",
            "movupd xmm15, [r12 + 240]",

            // Shuffle all in-place to separate colors and indices
            "pshufd xmm0,  xmm0,  0xD8",  // 11011000b - reorganize for color/index split
            "pshufd xmm1,  xmm1,  0xD8",
            "pshufd xmm2,  xmm2,  0xD8",
            "pshufd xmm3,  xmm3,  0xD8",
            "pshufd xmm4,  xmm4,  0xD8",
            "pshufd xmm5,  xmm5,  0xD8",
            "pshufd xmm6,  xmm6,  0xD8",
            "pshufd xmm7,  xmm7,  0xD8",
            "pshufd xmm8,  xmm8,  0xD8",
            "pshufd xmm9,  xmm9,  0xD8",
            "pshufd xmm10, xmm10, 0xD8",
            "pshufd xmm11, xmm11, 0xD8",
            "pshufd xmm12, xmm12, 0xD8",
            "pshufd xmm13, xmm13, 0xD8",
            "pshufd xmm14, xmm14, 0xD8",
            "pshufd xmm15, xmm15, 0xD8",

            // Store colors (lower halves)
            "movupd [r13],      xmm0",
            "movupd [r13 + 16], xmm1",
            "movupd [r13 + 32], xmm2",
            "movupd [r13 + 48], xmm3",
            "movupd [r13 + 64], xmm4",
            "movupd [r13 + 80], xmm5",
            "movupd [r13 + 96], xmm6",
            "movupd [r13 + 112], xmm7",
            "movupd [r13 + 128], xmm8",
            "movupd [r13 + 144], xmm9",
            "movupd [r13 + 160], xmm10",
            "movupd [r13 + 176], xmm11",
            "movupd [r13 + 192], xmm12",
            "movupd [r13 + 208], xmm13",
            "movupd [r13 + 224], xmm14",
            "movupd [r13 + 240], xmm15",

            // Store indices (upper halves)
            "movupd [r14],      xmm0",
            "movupd [r14 + 16], xmm1",
            "movupd [r14 + 32], xmm2",
            "movupd [r14 + 48], xmm3",
            "movupd [r14 + 64], xmm4",
            "movupd [r14 + 80], xmm5",
            "movupd [r14 + 96], xmm6",
            "movupd [r14 + 112], xmm7",
            "movupd [r14 + 128], xmm8",
            "movupd [r14 + 144], xmm9",
            "movupd [r14 + 160], xmm10",
            "movupd [r14 + 176], xmm11",
            "movupd [r14 + 192], xmm12",
            "movupd [r14 + 208], xmm13",
            "movupd [r14 + 224], xmm14",
            "movupd [r14 + 240], xmm15",

            // Update pointers
            "add r12, 256",  // src += 16 * 16
            "add r13, 256",  // colors_ptr += 16 * 16
            "add r14, 256",  // indices_ptr += 16 * 16

            // Compare against end address and loop if not done
            "cmp r12, rbx",
            "jb 2b",

            // Restore preserved registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",

            src = in(reg) input.as_ptr(),
            dst = in(reg) output.as_mut_ptr(),
            len = in(reg) input.len(),
            len_half = in(reg) input.len() / 2,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    enum Method {
        CpuPortable64,
        Sse2,
    }

    fn transform(method: Method, input: &[u8], output: &mut [u8]) {
        match method {
            Method::CpuPortable64 => unsafe { transform_64bit_portable(input, output) },
            Method::Sse2 => unsafe { transform_64bit_sse2(input, output) },
        }
    }

    // Helper to generate test data of specified size (in blocks)
    fn generate_test_data(num_blocks: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(num_blocks * 8);

        for i in 0..num_blocks {
            // Colors: Sequential bytes 1-64 (ensuring no overlap with indices)
            data.extend_from_slice(&[
                (1 + i * 4) as u8,
                (2 + i * 4) as u8,
                (3 + i * 4) as u8,
                (4 + i * 4) as u8,
            ]);

            // Indices: Sequential bytes 128-191 (well separated from colors)
            data.extend_from_slice(&[
                (128 + i * 4) as u8,
                (129 + i * 4) as u8,
                (130 + i * 4) as u8,
                (131 + i * 4) as u8,
            ]);
        }
        data
    }

    #[test]
    fn test_transform_cpu() {
        let input: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 colours
            0x10, 0x11, 0x12, 0x13, // block 1 indices
            0x04, 0x05, 0x06, 0x07, // block 2 colours
            0x14, 0x15, 0x16, 0x17, // block 2 indices
            0x08, 0x09, 0x0A, 0x0B, // block 3 colours
            0x18, 0x19, 0x1A, 0x1B, // block 3 indices
        ];
        let mut output = vec![0u8; 24];
        transform(Method::CpuPortable64, &input, &mut output);
        assert_eq!(
            output,
            vec![
                0x00, 0x01, 0x02, 0x03, // colours: block 1
                0x04, 0x05, 0x06, 0x07, // colours: block 2
                0x08, 0x09, 0x0A, 0x0B, // colours: block 3
                0x10, 0x11, 0x12, 0x13, // indices: block 1
                0x14, 0x15, 0x16, 0x17, // indices: block 2
                0x18, 0x19, 0x1A, 0x1B, // indices: block 3
            ]
        );
    }

    #[cfg(target_arch = "x86_64")]
    #[rstest]
    #[case::one_unroll(32)] // 256 bytes - tests single unroll iteration
    #[case::many_unrolls(256)] // 2KB - tests 8 unroll iterations
    fn test_sse2_against_portable(#[case] num_blocks: usize) {
        // Note: SSE2 is part of the "x86_64" target architecture spec
        //       so no need to test.
        let input = generate_test_data(num_blocks);
        let mut output_portable = vec![0u8; input.len()];
        let mut output_sse = vec![0u8; input.len()];

        // Run both implementations
        transform(Method::CpuPortable64, &input, &mut output_portable);
        transform(Method::Sse2, &input, &mut output_sse);

        // Compare results
        assert_eq!(
            output_portable, output_sse,
            "SSE2 and portable implementations produced different results for {} blocks.\n\
             First differing block will have predictable values:\n\
             Colors: Sequential 1-4 + (block_num * 4)\n\
             Indices: Sequential 128-131 + (block_num * 4)",
            num_blocks
        );
    }
}

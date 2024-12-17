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
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
#[inline(never)]
pub unsafe fn transform_64bit_portable(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    unsafe {
        let max_ptr = input_ptr.add(len) as *mut u64;
        let mut input_ptr = input_ptr as *mut u64;

        // Split output into color and index sections
        let mut colours_ptr = output_ptr as *mut u32;
        let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

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

/// Transform into separated color/index format preserving byte order
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
#[inline(never)]
pub unsafe fn transform_64bit_portable_unroll(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    unsafe {
        let max_ptr = input_ptr.add(len) as *mut u64;
        let mut input_ptr = input_ptr as *mut u64;

        // Split output into color and index sections
        let mut colours_ptr = output_ptr as *mut u32;
        let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

        while input_ptr.add(3) < max_ptr {
            // Load 4 blocks at once
            let curr1 = *input_ptr;
            let curr2 = *input_ptr.add(1);
            let curr3 = *input_ptr.add(2);
            let curr4 = *input_ptr.add(3);

            // Split into colours and indices
            let color1 = curr1 as u32;
            let color2 = curr2 as u32;
            let color3 = curr3 as u32;
            let color4 = curr4 as u32;

            // Store all colors
            *colours_ptr = color1;
            *colours_ptr.add(1) = color2;
            *colours_ptr.add(2) = color3;
            *colours_ptr.add(3) = color4;

            let index1 = (curr1 >> 32) as u32;
            let index2 = (curr2 >> 32) as u32;
            let index3 = (curr3 >> 32) as u32;
            let index4 = (curr4 >> 32) as u32;

            // Store all indices
            *indices_ptr = index1;
            *indices_ptr.add(1) = index2;
            *indices_ptr.add(2) = index3;
            *indices_ptr.add(3) = index4;

            // Update pointers
            input_ptr = input_ptr.add(4);
            colours_ptr = colours_ptr.add(4);
            indices_ptr = indices_ptr.add(4);
        }
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for SSE operations
#[cfg(target_arch = "x86_64")]
#[inline(never)]
pub unsafe fn transform_64bit_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

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

            // Load 8 blocks (128 bytes)
            "movdqa xmm0, [r12]",
            "movdqa xmm1, [r12 + 16]",
            "movdqa xmm2, [r12 + 32]",
            "movdqa xmm3, [r12 + 48]",
            "movdqa xmm4, [r12 + 64]",
            "movdqa xmm5, [r12 + 80]",
            "movdqa xmm6, [r12 + 96]",
            "movdqa xmm7, [r12 + 112]",

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
            "punpcklqdq xmm8, xmm1",     // colors 0,1
            "punpckhqdq xmm2, xmm3",     // indices 2,3
            "punpcklqdq xmm9, xmm3",     // colors 2,3
            "punpckhqdq xmm4, xmm5",     // indices 4,5
            "punpcklqdq xmm10, xmm5",    // colors 4,5
            "punpckhqdq xmm6, xmm7",     // indices 6,7
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
            "add r12, 128",  // src += 8 * 16
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
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    enum Method {
        CpuPortable64,
        Sse2,
        //Avx2,
    }

    fn transform(method: Method, input: &[u8], output: &mut [u8]) {
        unsafe {
            match method {
                Method::CpuPortable64 => {
                    transform_64bit_portable(input.as_ptr(), output.as_mut_ptr(), input.len())
                }
                Method::Sse2 => {
                    transform_64bit_sse2(input.as_ptr(), output.as_mut_ptr(), input.len())
                }
            }
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

    /*
    #[cfg(target_arch = "x86_64")]
    #[rstest]
    #[case::one_unroll(64)] // 512 bytes - tests single unroll iteration
    #[case::many_unrolls(512)] // 4KB - tests 8 unroll iterations
    fn test_avx2_against_portable(#[case] num_blocks: usize) {
        let input = generate_test_data(num_blocks);
        let mut output_portable = vec![0u8; input.len()];
        let mut output_avx = vec![0u8; input.len()];

        // Run both implementations
        transform(Method::CpuPortable64, &input, &mut output_portable);
        transform(Method::Avx2, &input, &mut output_avx);

        // Compare results
        assert_eq!(
            output_portable, output_avx,
            "AVX2 and portable implementations produced different results for {} blocks.\n\
             First differing block will have predictable values:\n\
             Colors: Sequential 1-4 + (block_num * 4)\n\
             Indices: Sequential 128-131 + (block_num * 4)",
            num_blocks
        );
    }
    */
}

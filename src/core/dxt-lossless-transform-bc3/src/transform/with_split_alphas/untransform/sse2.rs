#![allow(missing_docs)]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::generic::untransform_with_split_alphas as generic_untransform;

/// Untransforms BC3 block data from five separate arrays using SSE2 instructions.
///
/// # Arguments
///
/// * `alpha0_ptr` - Pointer to the input buffer containing alpha0 values (1 byte per block).
/// * `alpha1_ptr` - Pointer to the input buffer containing alpha1 values (1 byte per block).
/// * `alpha_indices_ptr` - Pointer to the input buffer containing packed alpha indices (6 bytes per block).
/// * `colors_ptr` - Pointer to the input buffer containing color endpoint pairs (4 bytes per block).
/// * `color_indices_ptr` - Pointer to the input buffer containing color indices (4 bytes per block).
/// * `output_ptr` - Pointer to the output buffer where the reconstructed BC3 blocks (16 bytes per block) will be written.
/// * `block_count` - The number of BC3 blocks to process.
///
/// # Safety
///
/// - `alpha0_ptr` must be valid for reads of `block_count` bytes.
/// - `alpha1_ptr` must be valid for reads of `block_count` bytes.
/// - `alpha_indices_ptr` must be valid for reads of `block_count * 6` bytes.
/// - `colors_ptr` must be valid for reads of `block_count * 4` bytes.
/// - `color_indices_ptr` must be valid for reads of `block_count * 4` bytes.
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes.
/// - Pointers do not need to be aligned; unaligned loads/stores are used.
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn untransform_with_split_alphas_sse2(
    mut alpha0_ptr: *const u8,
    mut alpha1_ptr: *const u8,
    mut alpha_indices_ptr: *const u16,
    mut colors_ptr: *const u32,
    mut color_indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Process 4 blocks (64 bytes output) per iteration
    const BLOCKS_PER_ITERATION: usize = 4;
    let aligned_block_count = block_count - (block_count % BLOCKS_PER_ITERATION);

    if aligned_block_count > 0 {
        let alpha0_end_ptr = alpha0_ptr.add(aligned_block_count);

        while alpha0_ptr < alpha0_end_ptr {
            // Read alpha0 and alpha1 bytes for 4 blocks (4 bytes each)
            let alpha0_bytes = (alpha0_ptr as *const u32).read_unaligned();
            let alpha1_bytes = (alpha1_ptr as *const u32).read_unaligned();

            // Interleave alpha0 and alpha1 bytes to form [alpha0, alpha1] pairs for each block
            // Block 0: alpha0[0], alpha1[0] -> offset 0
            // Block 1: alpha0[1], alpha1[1] -> offset 16
            // Block 2: alpha0[2], alpha1[2] -> offset 32
            // Block 3: alpha0[3], alpha1[3] -> offset 48
            write_u8(output_ptr, 0, shift_u32_u8(alpha0_bytes, 0));
            write_u8(output_ptr, 1, shift_u32_u8(alpha1_bytes, 0));
            write_u8(output_ptr, 16, shift_u32_u8(alpha0_bytes, 8));
            write_u8(output_ptr, 17, shift_u32_u8(alpha1_bytes, 8));
            write_u8(output_ptr, 32, shift_u32_u8(alpha0_bytes, 16));
            write_u8(output_ptr, 33, shift_u32_u8(alpha1_bytes, 16));
            write_u8(output_ptr, 48, shift_u32_u8(alpha0_bytes, 24));
            write_u8(output_ptr, 49, shift_u32_u8(alpha1_bytes, 24));

            // Handle alpha indices - read 24 bytes (6 bytes × 4 blocks) using u64 reads
            // Similar pattern to standard SSE2 untransform
            let alpha_indices_ptr_u64 = alpha_indices_ptr as *const u64;

            let alpha_bits_0 = alpha_indices_ptr_u64.read_unaligned();
            write_u16(output_ptr, 2, shift_u64_u16(alpha_bits_0, 0));
            write_u32(output_ptr, 4, shift_u64_u32(alpha_bits_0, 16)); // block 0 complete (6/6 bytes)
            write_u16(output_ptr, 18, shift_u64_u16(alpha_bits_0, 48)); // block 1 start (2/6 bytes)

            let alpha_bits_1 = alpha_indices_ptr_u64.add(1).read_unaligned();
            write_u32(output_ptr, 20, shift_u64_u32(alpha_bits_1, 0)); // block 1 complete (6/6 bytes)
            write_u32(output_ptr, 34, shift_u64_u32(alpha_bits_1, 32)); // block 2 start (4/6 bytes)

            let alpha_bits_2 = alpha_indices_ptr_u64.add(2).read_unaligned();
            write_u16(output_ptr, 38, shift_u64_u16(alpha_bits_2, 0)); // block 2 complete (6/6 bytes)
            write_u64(output_ptr, 50, alpha_bits_2 >> 16); // block 3 atomic write (overwrites past end, fixed by SIMD write below)

            alpha_indices_ptr = alpha_indices_ptr.add(12); // 24 bytes = 12 u16s

            // Load and interleave colors/indices using SSE2
            let colors = _mm_loadu_si128(colors_ptr as *const __m128i);
            let indices = _mm_loadu_si128(color_indices_ptr as *const __m128i);

            let low = _mm_unpacklo_epi32(colors, indices);
            let high = _mm_unpackhi_epi32(colors, indices);

            // Store interleaved colors+indices for all 4 blocks
            _mm_storel_epi64(output_ptr.add(8) as *mut __m128i, low);
            _mm_storeh_pd(output_ptr.add(24) as *mut f64, _mm_castsi128_pd(low));
            _mm_storel_epi64(output_ptr.add(40) as *mut __m128i, high);
            _mm_storeh_pd(output_ptr.add(56) as *mut f64, _mm_castsi128_pd(high));

            // Advance pointers
            alpha0_ptr = alpha0_ptr.add(BLOCKS_PER_ITERATION);
            alpha1_ptr = alpha1_ptr.add(BLOCKS_PER_ITERATION);
            colors_ptr = colors_ptr.add(BLOCKS_PER_ITERATION);
            color_indices_ptr = color_indices_ptr.add(BLOCKS_PER_ITERATION);
            output_ptr = output_ptr.add(BLOCKS_PER_ITERATION * 16);
        }
    }

    // Process remaining blocks (< 4) using generic implementation
    generic_untransform(
        alpha0_ptr,
        alpha1_ptr,
        alpha_indices_ptr,
        colors_ptr,
        color_indices_ptr,
        output_ptr,
        block_count - aligned_block_count,
    );
}

#[inline(always)]
unsafe fn write_u8(ptr: *mut u8, offset: usize, value: u8) {
    ptr.add(offset).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u16(ptr: *mut u8, offset: usize, value: u16) {
    (ptr.add(offset) as *mut u16).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u32(ptr: *mut u8, offset: usize, value: u32) {
    (ptr.add(offset) as *mut u32).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u64(ptr: *mut u8, offset: usize, value: u64) {
    (ptr.add(offset) as *mut u64).write_unaligned(value);
}

#[inline(always)]
fn shift_u32_u8(value: u32, shift: usize) -> u8 {
    (value >> shift) as u8
}

#[inline(always)]
fn shift_u64_u16(value: u64, shift: usize) -> u16 {
    (value >> shift) as u16
}

#[inline(always)]
fn shift_u64_u32(value: u64, shift: usize) -> u32 {
    (value >> shift) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_with_split_alphas_sse2, "sse2", 8)]
    fn test_sse2_unaligned(
        #[case] untransform_fn: SplitAlphasUntransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For SSE2: processes 64 bytes (4 blocks) per iteration, so max_blocks = 64 bytes × 2 ÷ 16 = 8
        run_with_split_alphas_untransform_unaligned_test(untransform_fn, max_blocks, impl_name);
    }
}

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
/// - colors_out must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_out must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8
/// - The color and index buffers must not overlap with each other or the input buffer
#[allow(unused_assignments)] // no feature for 512
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
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
                colors_ptr = inout(reg) colors_out,
                indices_ptr = inout(reg) indices_out,
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
    u32_with_separate_pointers(input_ptr, colors_out, indices_out, remaining);
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
    use crate::test_prelude::*;
    use crate::transforms::standard::untransform::untransform;

    #[rstest]
    #[case(permute_512, "avx512 permute")]
    #[case(permute_512_unroll_2, "avx512 permute unroll 2")]
    fn avx512_transform_roundtrip(#[case] permute_fn: unsafe fn(*const u8, *mut u8, usize), #[case] impl_name: &str) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut transformed = vec![0u8; len];
            let mut reconstructed = vec![0u8; len];
            
            unsafe {
                permute_fn(input.as_ptr(), transformed.as_mut_ptr(), len);
                untransform(transformed.as_ptr(), reconstructed.as_mut_ptr(), len);
            }
            
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "Mismatch {impl_name} roundtrip for {num_blocks} blocks",
            );
        }
    }

    #[rstest]
    #[case(permute_512_with_separate_pointers, "avx512 separate pointers")]
    #[case(permute_512_unroll_2_with_separate_pointers, "avx512 unroll 2 separate pointers")]
    fn avx512_separate_pointers_roundtrip(
        #[case] permute_fn: unsafe fn(*const u8, *mut u32, *mut u32, usize), 
        #[case] impl_name: &str
    ) {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut colors_sep = allocate_align_64(len / 2).unwrap();
            let mut indices_sep = allocate_align_64(len / 2).unwrap();
            let mut reconstructed = vec![0u8; len];

            unsafe {
                // Transform: separate pointers variant
                permute_fn(
                    input.as_ptr(),
                    colors_sep.as_mut_ptr() as *mut u32,
                    indices_sep.as_mut_ptr() as *mut u32,
                    len,
                );

                // Reconstruct by combining colors and indices back into standard format
                let mut combined = vec![0u8; len];
                combined[0..len / 2].copy_from_slice(colors_sep.as_slice());
                combined[len / 2..].copy_from_slice(indices_sep.as_slice());

                // Untransform back to original format
                untransform(combined.as_ptr(), reconstructed.as_mut_ptr(), len);
            }

            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "{impl_name} roundtrip failed for {num_blocks} blocks"
            );
        }
    }
}

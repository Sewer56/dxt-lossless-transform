use crate::split_blocks::unsplit::portable32::u32_detransform_with_separate_pointers;
use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 32-byte blocks as possible
    let aligned_len = len - (len % 32);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load colors and indices (16 bytes each)
                "movdqu {xmm0}, [{colors_ptr}]",    // colors
                "add {colors_ptr}, 16",
                "movdqu {xmm1}, [{indices_ptr}]",   // indices
                "add {indices_ptr}, 16",

                // Interleave the 32-bit values
                "movaps {xmm2}, {xmm0}",
                "unpcklps {xmm0}, {xmm1}",    // Low half: color0,index0,color1,index1
                "unpckhps {xmm2}, {xmm1}",    // High half: color2,index2,color3,index3

                // Store the results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "add {dst_ptr}, 32",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
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
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;
    unpck_detransform_unroll_2_with_components(output_ptr, len, indices_ptr, colors_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_2_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_ptr: *const u8,
    mut colors_ptr: *const u8,
) {
    debug_assert!(len % 8 == 0);
    let aligned_len = len - (len % 64);
    let colors_aligned_end = colors_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load all colors and indices (32 bytes each)
                "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
                "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
                "add {colors_ptr}, 32",
                "movdqu {xmm1}, [{indices_ptr}]",     // indices 0
                "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
                "add {indices_ptr}, 32",

                // Save copies for high parts
                "movaps {xmm2}, {xmm0}", // colors 0 copy
                "movaps {xmm5}, {xmm3}", // colors 1 copy

                // Unpack all blocks
                "punpckldq {xmm0}, {xmm1}", // color0,index0,color1,index1
                "punpckldq {xmm3}, {xmm4}", // color4,index4,color5,index5
                "punpckhdq {xmm2}, {xmm1}", // color2,index2,color3,index3
                "punpckhdq {xmm5}, {xmm4}", // color6,index6,color7,index7

                // Store all results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "movdqu [{dst_ptr} + 32], {xmm3}",
                "movdqu [{dst_ptr} + 48], {xmm5}",
                "add {dst_ptr}, 64",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
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
        u32_detransform_with_separate_pointers(
            colors_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load all colors (64 bytes)
                "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
                "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
                "movdqu {xmm6}, [{colors_ptr} + 32]", // colors 2
                "movdqu {xmm9}, [{colors_ptr} + 48]", // colors 3
                "add {colors_ptr}, 64",

                // Load all indices (64 bytes)
                "movdqu {xmm1}, [{indices_ptr}]",      // indices 0
                "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
                "movdqu {xmm7}, [{indices_ptr} + 32]", // indices 2
                "movdqu {xmm10}, [{indices_ptr} + 48]", // indices 3
                "add {indices_ptr}, 64",

                // Save copies for high parts
                "movaps {xmm2}, {xmm0}",  // colors 0 copy
                "movaps {xmm5}, {xmm3}",  // colors 1 copy
                "movaps {xmm8}, {xmm6}",  // colors 2 copy
                "movaps {xmm11}, {xmm9}", // colors 3 copy

                // Unpack all blocks
                "unpcklps {xmm0}, {xmm1}",
                "unpckhps {xmm2}, {xmm1}",
                "unpcklps {xmm3}, {xmm4}",
                "unpckhps {xmm5}, {xmm4}",
                "unpcklps {xmm6}, {xmm7}",
                "unpckhps {xmm8}, {xmm7}",
                "unpcklps {xmm9}, {xmm10}",
                "unpckhps {xmm11}, {xmm10}",

                // Store all results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "movdqu [{dst_ptr} + 32], {xmm3}",
                "movdqu [{dst_ptr} + 48], {xmm5}",
                "movdqu [{dst_ptr} + 64], {xmm6}",
                "movdqu [{dst_ptr} + 80], {xmm8}",
                "movdqu [{dst_ptr} + 96], {xmm9}",
                "movdqu [{dst_ptr} + 112], {xmm11}",
                "add {dst_ptr}, 128",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                xmm0 = out(xmm_reg) _,
                xmm1 = out(xmm_reg) _,
                xmm2 = out(xmm_reg) _,
                xmm3 = out(xmm_reg) _,
                xmm4 = out(xmm_reg) _,
                xmm5 = out(xmm_reg) _,
                xmm6 = out(xmm_reg) _,
                xmm7 = out(xmm_reg) _,
                xmm8 = out(xmm_reg) _,
                xmm9 = out(xmm_reg) _,
                xmm10 = out(xmm_reg) _,
                xmm11 = out(xmm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(unpck_detransform, "unpck")]
    #[case(unpck_detransform_unroll_2, "unpck_unroll_2")]
    #[cfg_attr(
        target_arch = "x86_64",
        case(unpck_detransform_unroll_4, "unpck_unroll_4")
    )]
    fn test_sse2_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform using standard implementation
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Reconstruct using the implementation being tested
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                &format!("{} (aligned)", impl_name),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(unpck_detransform, "unpck")]
    #[case(unpck_detransform_unroll_2, "unpck_unroll_2")]
    #[cfg_attr(
        target_arch = "x86_64",
        case(unpck_detransform_unroll_4, "unpck_unroll_4")
    )]
    fn test_sse2_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            unsafe {
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);

            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed_unaligned.as_ptr().add(1),
                    reconstructed.as_mut_ptr().add(1),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{} (unaligned)", impl_name),
                num_blocks,
            );
        }
    }
}

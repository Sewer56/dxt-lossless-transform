/*
 * BC1 Block Rearrangement Optimization Explanation
 * =================================================
 *
 * Original sequential BC1 data layout:
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

pub mod detransform;
pub mod transform;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_bc1_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let is_32_aligned = input_ptr as usize % 32 == 0 && output_ptr as usize % 32 == 0;
    let is_16_aligned = input_ptr as usize % 16 == 0 && output_ptr as usize % 16 == 0;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        let avx2 = std::is_x86_feature_detected!("avx2");
        let sse2 = std::is_x86_feature_detected!("sse2");

        if avx2 && len % 256 == 0 && is_32_aligned {
            transform::shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if sse2 && len % 64 == 0 && is_16_aligned {
            transform::shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_feature = "avx2")]
        if len % 256 == 0 && is_32_aligned {
            transform::shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_feature = "sse2")]
        if len % 64 == 0 && is_16_aligned {
            transform::shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    transform::u32(input_ptr, output_ptr, len)
}

/// Transform BC1 data from standard interleaved format to separated color/index format.
///
/// This function selects the best available implementation based on CPU features:
/// 1. AVX2 shuffle_permute_unroll_4 on x86_64 systems
/// 2. AVX2 shuffle_permute_unroll_2 on x86 systems
/// 3. SSE2 shufps_unroll_4 if available
/// 4. Portable 32-bit implementation as fallback
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_bc1_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        transform::u32(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_bc1_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        let avx2 = std::is_x86_feature_detected!("avx2");
        let sse2 = std::is_x86_feature_detected!("sse2");

        if avx2 && len % 256 == 0 {
            #[cfg(target_arch = "x86_64")]
            {
                detransform::avx2::unpck_detransform_unroll_4(input_ptr, output_ptr, len);
                return;
            }
            #[cfg(target_arch = "x86")]
            {
                detransform::avx2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
                return;
            }
        }

        if sse2 && len % 128 == 0 {
            #[cfg(target_arch = "x86_64")]
            {
                detransform::sse2::unpck_detransform_unroll_4(input_ptr, output_ptr, len);
                return;
            }
            #[cfg(target_arch = "x86")]
            {
                detransform::sse2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
                return;
            }
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_feature = "avx2")]
        if len % 256 == 0 {
            #[cfg(target_arch = "x86_64")]
            {
                detransform::avx2::unpck_detransform_unroll_4(input_ptr, output_ptr, len);
                return;
            }
            #[cfg(target_arch = "x86")]
            {
                detransform::avx2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
                return;
            }
        }

        #[cfg(target_feature = "sse2")]
        if len % 128 == 0 {
            #[cfg(target_arch = "x86_64")]
            {
                detransform::sse2::unpck_detransform_unroll_4(input_ptr, output_ptr, len);
                return;
            }
            #[cfg(target_arch = "x86")]
            {
                detransform::sse2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
                return;
            }
        }
    }

    // Fallback to portable implementation
    detransform::u32_detransform(input_ptr, output_ptr, len)
}

/// Transform BC1 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on CPU features:
/// 1. AVX2 unpck_detransform_unroll_4 on x86_64 systems
/// 2. AVX2 unpck_detransform_unroll_2 on x86 systems
/// 3. SSE2 unpck_detransform_unroll_4 if available
/// 4. Portable 32-bit implementation as fallback
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - input_ptr and output_ptr must be 64-byte aligned for optimal performance
/// - pointers must be properly aligned for the operation
#[inline]
pub unsafe fn untransform_bc1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    debug_assert_eq!(
        input_ptr as usize % 64,
        0,
        "input_ptr must be 64-byte aligned"
    );
    debug_assert_eq!(
        output_ptr as usize % 64,
        0,
        "output_ptr must be 64-byte aligned"
    );

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_bc1_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        detransform::u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc1::transform::tests::{
        generate_bc1_test_data, transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    #[rstest]
    #[case::min_size(32)] // 256 bytes - minimum size for most operations
    #[case::one_unroll(64)] // 512 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_transform_untransform(#[case] num_blocks: usize) {
        let input = generate_bc1_test_data(num_blocks);
        let mut transformed = allocate_align_64(input.len());
        let mut reconstructed = allocate_align_64(input.len());
        let mut reference = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), reference.as_mut_slice());

        unsafe {
            // Test transform
            transform_bc1(input.as_ptr(), transformed.as_mut_ptr(), input.len());
            assert_eq!(
                transformed.as_slice(),
                reference.as_slice(),
                "transform_bc1 produced different results than reference for {} blocks",
                num_blocks
            );

            // Test untransform
            untransform_bc1(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "untransform_bc1 failed to reconstruct original data for {} blocks",
                num_blocks
            );
        }
    }
}

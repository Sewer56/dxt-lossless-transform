/*
 * BC2/DXT3 Block Rearrangement Optimization Explanation
 * ==================================================
 *
 * Original sequential BC2 data layout:
 * 8 bytes of alpha values followed by two 16-bit colours (4 bytes) and 4 bytes of color indices:
 *
 * Address: 0       8       12      16  16      24      28      32
 *          +-------+-------+-------+   +-------+-------+--------+
 * Data:    |A0-A15 | C0-C1 | I0-I15 |  |A16-A31| C2-C3 | I6-I31 |
 *          +-------+-------+-------+   +-------+-------+--------+
 *
 * Each 16-byte block contains:
 * - 8 bytes of explicit alpha (sixteen 4-bit alpha values)
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed color indices (sixteen 2-bit indices)
 *
 * Optimized layout separates alpha, colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } Alpha section
 * | A0    | A1    | A2    | ... | AN    |  } (8 bytes per block: 16x 4-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Alpha values often have high spatial coherency
 * 2. Color endpoints tend to be spatially coherent
 * 3. Index patterns often repeat across blocks
 * 4. Separating them allows better compression of each stream
 *
 * Key differences from BC1/DXT1:
 * - Blocks are 16 bytes instead of 8 bytes
 * - Includes explicit 4-bit alpha values (no alpha interpolation)
 * - No special "transparent black" color combinations
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
unsafe fn transform_bc2_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        let avx2 = std::is_x86_feature_detected!("avx2");
        let sse2 = std::is_x86_feature_detected!("sse2");

        if avx2 && len % 128 == 0 {
            transform::avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if sse2 && len % 128 == 0 {
            transform::sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        if sse2 && len % 64 == 0 {
            transform::sse2::shuffle_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_feature = "avx2")]
        if len % 128 == 0 {
            transform::avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_feature = "sse2")]
        #[cfg(target_arch = "x86_64")]
        if len % 128 == 0 {
            transform::sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_feature = "sse2")]
        #[cfg(target_arch = "x86")]
        if len % 64 == 0 {
            transform::sse2::shuffle_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    transform::u32(input_ptr, output_ptr, len)
}

/// Transform BC2 data from standard interleaved format to separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        transform::u32(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_bc2_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        let avx2 = std::is_x86_feature_detected!("avx2");
        let sse2 = std::is_x86_feature_detected!("sse2");

        if avx2 && len % 128 == 0 {
            detransform::avx2::avx2_shuffle(input_ptr, output_ptr, len);
        }

        if sse2 && len % 64 == 0 {
            detransform::sse2::shuffle(input_ptr, output_ptr, len);
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_feature = "avx2")]
        if len % 128 == 0 {
            detransform::avx2::avx2_shuffle(input_ptr, output_ptr, len);
        }

        #[cfg(target_feature = "sse2")]
        if len % 64 == 0 {
            detransform::sse2::shuffle(input_ptr, output_ptr, len);
        }
    }

    // Fallback to portable implementation
    detransform::u32_detransform(input_ptr, output_ptr, len)
}

/// Transform BC2 data from separated color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        detransform::u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bc2::transform::tests::{
        generate_bc2_test_data, transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    #[rstest]
    #[case::min_size(1)] // 8 bytes - minimum size
    #[case::min_size(2)] // 16 bytes - SSE Register
    #[case::min_size(4)] // 32 bytes - AVX Register
    #[case::min_size(8)] // 64 bytes - SSE Unrolled Operation
    #[case::min_size(16)] // 128 bytes - AVX Unrolled Operation
    #[case::min_size(32)] // 256 bytes - Multiple Unrolled Operations
    fn test_transform_untransform(#[case] num_blocks: usize) {
        let input = generate_bc2_test_data(num_blocks);
        let mut transformed = allocate_align_64(input.len());
        let mut reconstructed = allocate_align_64(input.len());
        let mut reference = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), reference.as_mut_slice());

        unsafe {
            // Test transform
            transform_bc2(input.as_ptr(), transformed.as_mut_ptr(), input.len());
            assert_eq!(
                transformed.as_slice(),
                reference.as_slice(),
                "transform_bc2 produced different results than reference for {} blocks",
                num_blocks
            );

            // Test untransform
            untransform_bc2(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "untransform_bc2 failed to reconstruct original data for {} blocks",
                num_blocks
            );
        }
    }
}

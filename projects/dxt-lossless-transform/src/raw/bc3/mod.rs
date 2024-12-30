/*
 * BC3/DXT5 Block Rearrangement Optimization Explanation
 * ==================================================
 *
 * Original sequential BC3 data layout:
 * 2 bytes of alpha endpoints followed by 6 bytes of alpha indices, then two 16-bit colours (4 bytes)
 * and 4 bytes of color indices:
 *
 * Address: 0       2       8       12      16  16      18      24      28      32
 *          +-------+-------+-------+-------+   +-------+-------+-------+-------+
 * Data:    |A0-A1  |AIdx0-47|C0-C1 |I0-I15 |  |A2-A3  |AIdx48-95|C2-C3 |I16-I31|
 *          +-------+-------+-------+-------+   +-------+-------+-------+-------+
 *
 * Each 16-byte block contains:
 * - 2 bytes of alpha endpoints (min/max alpha values for interpolation)
 * - 6 bytes of alpha indices (sixteen 3-bit indices for alpha interpolation)
 * - 4 bytes colours (2x RGB565 values)
 * - 4 bytes of packed color indices (sixteen 2-bit indices)
 *
 * Optimized layout separates alpha endpoints, alpha indices, colours and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } Alpha endpoints section
 * | A0-A1 | A2-A3 | A4-A5 | ... | AN    |  } (2 bytes per block: 2x 8-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Alpha indices section
 * |AI0-47 |AI48-95|  ...  | ... |AI N   |  } (6 bytes per block: 16x 3-bit)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Colours section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Alpha endpoints often have high spatial coherency
 * 2. Alpha index patterns tend to repeat across similar regions
 * 3. Color endpoints tend to be spatially coherent
 * 4. Color index patterns often repeat across blocks
 * 5. Separating them allows better compression of each stream
 */

pub mod detransform;
pub mod transform;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_bc3_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        let avx2 = std::is_x86_feature_detected!("avx2");

        if avx2 && len % 128 == 0 {
            transform::avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_feature = "avx2")]
        if len % 128 == 0 {
            transform::avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    transform::u32(input_ptr, output_ptr, len)
}

/// Transform bc3 data from standard interleaved format to separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_bc3_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        transform::u32(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_bc3_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        if len % 64 == 0 {
            detransform::sse2::u64_detransform_sse2(input_ptr, output_ptr, len);
        }
    }

    if len % 32 == 0 {
        #[cfg(target_arch = "x86_64")]
        {
            detransform::u64_detransform(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        {
            detransform::u32_detransform_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    detransform::u32_detransform(input_ptr, output_ptr, len);
}

/// Transform bc3 data from separated color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_bc3_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        detransform::u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc3::transform::tests::{
        generate_bc3_test_data, transform_with_reference_implementation,
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
        let input = generate_bc3_test_data(num_blocks);
        let mut transformed = allocate_align_64(input.len());
        let mut reconstructed = allocate_align_64(input.len());
        let mut reference = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), reference.as_mut_slice());

        unsafe {
            // Test transform
            transform_bc3(input.as_ptr(), transformed.as_mut_ptr(), input.len());
            assert_eq!(
                transformed.as_slice(),
                reference.as_slice(),
                "transform_bc3 produced different results than reference for {} blocks",
                num_blocks
            );

            // Test untransform
            untransform_bc3(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "untransform_bc3 failed to reconstruct original data for {} blocks",
                num_blocks
            );
        }
    }
}

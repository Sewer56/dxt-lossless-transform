mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;

#[cfg(feature = "bench")]
pub mod bench;

/// Split BC1 blocks from standard interleaved format to separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32(input_ptr, output_ptr, len)
    }
}

/// Split BC1 blocks from standard interleaved format to separate color and index pointers
/// using the best known implementation for the current CPU.
///
/// This variant allows direct output to separate buffers for colors and indices, which can
/// be useful when you need the components stored in different memory locations or with
/// different layouts than the standard contiguous separated format.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8 (BC1 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - The color and index buffers must not overlap with each other or the input buffer
#[inline]
pub(crate) unsafe fn transform_with_separate_pointers(
    input_ptr: *const u8,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_with_separate_pointers_x86(input_ptr, colors_ptr, indices_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        use dxt_lossless_transform_common::cpu_detect::*;

        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if has_avx512f() {
            avx512::permute_512(input_ptr, output_ptr, len);
            return;
        }

        if has_avx2() {
            avx2::shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if has_sse2() {
            sse2::shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_with_separate_pointers_x86(
    input_ptr: *const u8,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        use dxt_lossless_transform_common::cpu_detect::*;

        #[cfg(feature = "nightly")]
        if has_avx512f() {
            avx512::permute_512_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if has_avx2() {
            avx2::shuffle_permute_unroll_2_with_separate_pointers(
                input_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        if has_sse2() {
            sse2::shufps_unroll_4_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle_permute_unroll_2_with_separate_pointers(
                input_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::shufps_unroll_4_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len)
}

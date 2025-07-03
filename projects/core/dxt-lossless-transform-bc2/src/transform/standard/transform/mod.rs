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

/// Transform BC2 data from standard interleaved format to separated alpha/color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn split_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32(input_ptr, output_ptr, len)
    }
}

/// Transform BC2 data from standard interleaved format to separated alpha/color/index format
/// using separate pointers for each component section.
///
/// # Arguments
///
/// * `input_ptr` - Pointer to the input buffer containing interleaved BC2 block data
/// * `alphas_ptr` - Pointer to the output buffer for alpha data (8 bytes per block)
/// * `colors_ptr` - Pointer to the output buffer for color data (4 bytes per block)
/// * `indices_ptr` - Pointer to the output buffer for index data (4 bytes per block)
/// * `len` - The length of the input buffer in bytes
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `alphas_ptr` must be valid for writes of `len / 2` bytes
/// - `colors_ptr` must be valid for writes of `len / 4` bytes
/// - `indices_ptr` must be valid for writes of `len / 4` bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn split_blocks_with_separate_pointers(
    input_ptr: *const u8,
    alphas_ptr: *mut u64,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_with_separate_pointers_x86(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_bc2_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // Note(sewer): The AVX512 implementation is disabled because a bunch of CPUs throttle on it,
    // leading to it being slower.,
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::permute_512_v2(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512_v2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_with_separate_pointers_x86(
    input_ptr: *const u8,
    alphas_ptr: *mut u64,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    // Note(sewer): The AVX512 implementation is disabled because a bunch of CPUs throttle on it,
    // leading to it being slower.,
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::permute_512_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::shuffle_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v3_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v3_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
}

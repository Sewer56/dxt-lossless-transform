pub mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_vbmi(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_vbmi(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32(input_ptr, output_ptr, len)
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
pub unsafe fn transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32(input_ptr, output_ptr, len)
    }
}

// Re-export functions for benchmarking when the 'bench' feature is enabled
#[cfg(feature = "bench")]
pub mod bench;

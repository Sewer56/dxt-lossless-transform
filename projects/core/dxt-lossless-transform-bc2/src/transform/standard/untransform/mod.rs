mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_blocks_bc2_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        // disabled due to non-guaranteed performance on 32-bit
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::avx512_shuffle(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::avx2_shuffle(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        // disabled due to non-guaranteed performance on 32-bit
        if cfg!(target_feature = "avx512f") {
            avx512::avx512_shuffle(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::avx2_shuffle(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::shuffle(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_untransform(input_ptr, output_ptr, len)
}

/// Transform BC2 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_blocks_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_untransform(input_ptr, output_ptr, len)
    }
}

// Re-export functions for benchmarking when the 'bench' feature is enabled
#[cfg(feature = "bench")]
#[allow(clippy::missing_safety_doc)]
#[allow(missing_docs)]
pub mod bench {
    pub unsafe fn u32_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
        super::portable32::u32_untransform(input_ptr, output_ptr, len)
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn sse2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
        super::sse2::shuffle(input_ptr, output_ptr, len)
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
        super::avx2::avx2_shuffle(input_ptr, output_ptr, len)
    }

    #[cfg(feature = "nightly")]
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    pub unsafe fn avx512_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
        super::avx512::avx512_shuffle(input_ptr, output_ptr, len)
    }
}

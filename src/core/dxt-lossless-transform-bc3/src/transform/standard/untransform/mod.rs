pub mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512vbmi;

#[cfg(feature = "bench")]
pub mod bench;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            // On x86_64, use the 64-bit optimized path
            #[cfg(target_arch = "x86_64")]
            {
                avx512vbmi::avx512_untransform(input_ptr, output_ptr, len);
            }
            // On 32-bit x86, use 32-bit optimized path
            #[cfg(target_arch = "x86")]
            {
                avx512vbmi::avx512_untransform_32(input_ptr, output_ptr, len);
            }
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512vbmi") {
            // On x86_64, use the 64-bit optimized path
            #[cfg(target_arch = "x86_64")]
            {
                avx512vbmi::avx512_untransform(input_ptr, output_ptr, len);
            }
            // On 32-bit x86, use 32-bit optimized path
            #[cfg(target_arch = "x86")]
            {
                avx512vbmi::avx512_untransform_32(input_ptr, output_ptr, len);
            }
            return;
        }
    }

    // SSE2 is required by x86-64, so no check needed
    #[cfg(target_arch = "x86_64")]
    {
        sse2::u64_untransform_sse2(input_ptr, output_ptr, len);
    }

    // On i686, SSE2 is not guaranteed, so we need runtime detection
    #[cfg(target_arch = "x86")]
    {
        #[cfg(not(feature = "no-runtime-cpu-detection"))]
        {
            if dxt_lossless_transform_common::cpu_detect::has_sse2() {
                sse2::u32_untransform_sse2(input_ptr, output_ptr, len);
                return;
            }
        }

        #[cfg(feature = "no-runtime-cpu-detection")]
        {
            if cfg!(target_feature = "sse2") {
                sse2::u32_untransform_sse2(input_ptr, output_ptr, len);
                return;
            }
        }

        portable32::u32_untransform_v2(input_ptr, output_ptr, len);
    }
}

/// Transform bc3 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_untransform_v2(input_ptr, output_ptr, len)
    }
}

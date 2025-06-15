pub mod portable;
pub use portable::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_blocks_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_detransform(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_arch = "x86_64")]
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_detransform(input_ptr, output_ptr, len);
            return;
        }
    }

    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        sse2::u64_detransform_sse2(input_ptr, output_ptr, len);
    }

    #[cfg(target_arch = "x86")]
    {
        u32_detransform_v2(input_ptr, output_ptr, len);
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
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_blocks_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform_v2(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_block_with_separate_pointers_x86(
    alpha_byte_ptr: *const u8,
    alpha_bit_ptr: *const u8,
    color_byte_ptr: *const u8,
    index_byte_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_detransform_separate_components(
                alpha_byte_ptr,
                alpha_bit_ptr,
                color_byte_ptr,
                index_byte_ptr,
                output_ptr,
                len,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_arch = "x86_64")]
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_detransform_separate_components(
                alpha_byte_ptr,
                alpha_bit_ptr,
                color_byte_ptr,
                index_byte_ptr,
                output_ptr,
                len,
            );
            return;
        }
    }

    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        sse2::u64_detransform_sse2_separate_components(
            alpha_byte_ptr as *const u64,
            alpha_bit_ptr as *const u64,
            color_byte_ptr as *const core::arch::x86_64::__m128i,
            index_byte_ptr as *const core::arch::x86_64::__m128i,
            output_ptr,
            len,
        );
    }

    #[cfg(target_arch = "x86")]
    {
        portable::u32_detransform_with_separate_pointers(
            alpha_byte_ptr as *const u16,
            alpha_bit_ptr as *const u16,
            color_byte_ptr as *const u32,
            index_byte_ptr as *const u32,
            output_ptr,
            len,
        );
    }
}

/// Unsplit BC3 blocks, putting them back into standard interleaved format from separated component pointers
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - alpha_byte_ptr must be valid for reads of len * 2 / 16 bytes (alpha endpoints)
/// - alpha_bit_ptr must be valid for reads of len * 6 / 16 bytes (alpha indices)
/// - color_byte_ptr must be valid for reads of len * 4 / 16 bytes (color endpoints)
/// - index_byte_ptr must be valid for reads of len * 4 / 16 bytes (color indices)
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_block_with_separate_pointers(
    alpha_byte_ptr: *const u8,
    alpha_bit_ptr: *const u8,
    color_byte_ptr: *const u8,
    index_byte_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_block_with_separate_pointers_x86(
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_byte_ptr,
            index_byte_ptr,
            output_ptr,
            len,
        )
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        // Cast pointers to types expected by portable implementation
        portable::u32_detransform_with_separate_pointers(
            alpha_byte_ptr as *const u16,
            alpha_bit_ptr as *const u16,
            color_byte_ptr as *const u32,
            index_byte_ptr as *const u32,
            output_ptr,
            len,
        )
    }
}

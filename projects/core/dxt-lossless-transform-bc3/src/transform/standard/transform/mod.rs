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

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[allow(dead_code)] // Public API not yet exposed; kept for future use.
#[inline(always)]
unsafe fn transform_with_separate_pointers_x86(
    input_ptr: *const u8,
    alpha_byte_ptr: *mut u16,
    alpha_bit_ptr: *mut u16,
    color_ptr: *mut u32,
    index_ptr: *mut u32,
    len: usize,
) {
    let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_vbmi_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr as *mut u8,
                alpha_bit_ptr as *mut u8,
                color_ptr as *mut u8,
                index_ptr as *mut u8,
                len,
            );
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::u32_avx2_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr,
                alpha_bit_ptr as *mut u8,
                color_ptr,
                index_ptr,
                alpha_byte_end_ptr,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_vbmi_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr as *mut u8,
                alpha_bit_ptr as *mut u8,
                color_ptr as *mut u8,
                index_ptr as *mut u8,
                len,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::u32_avx2_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr,
                alpha_bit_ptr as *mut u8,
                color_ptr,
                index_ptr,
                alpha_byte_end_ptr,
            );
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_with_separate_endpoints(
        input_ptr,
        alpha_byte_ptr,
        alpha_bit_ptr,
        color_ptr,
        index_ptr,
        alpha_byte_end_ptr,
    )
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

/// Transform BC3 data from standard interleaved format to separated component format
/// using separate pointers for each component section.
///
/// # Arguments
///
/// * `input_ptr` - Pointer to the input buffer containing interleaved BC3 block data
/// * `alpha_byte_ptr` - Pointer to the output buffer for alpha endpoint data (2 bytes per block)
/// * `alpha_bit_ptr` - Pointer to the output buffer for alpha index data (6 bytes per block)  
/// * `color_ptr` - Pointer to the output buffer for color endpoint data (4 bytes per block)
/// * `index_ptr` - Pointer to the output buffer for color index data (4 bytes per block)
/// * `len` - The length of the input buffer in bytes
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `alpha_byte_ptr` must be valid for writes of `len * 2 / 16` bytes
/// - `alpha_bit_ptr` must be valid for writes of `len * 6 / 16` bytes
/// - `color_ptr` must be valid for writes of `len * 4 / 16` bytes
/// - `index_ptr` must be valid for writes of `len * 4 / 16` bytes
/// - `len` must be divisible by 16 (BC3 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - The component buffers must not overlap with each other or the input buffer
#[allow(dead_code)] // Public API not yet exposed; kept for future use.
#[inline]
pub unsafe fn transform_with_separate_pointers(
    input_ptr: *const u8,
    alpha_byte_ptr: *mut u16,
    alpha_bit_ptr: *mut u16,
    color_ptr: *mut u32,
    index_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len.is_multiple_of(16));

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_with_separate_pointers_x86(
            input_ptr,
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_ptr,
            index_ptr,
            len,
        )
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
        portable32::u32_with_separate_endpoints(
            input_ptr,
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_ptr,
            index_ptr,
            alpha_byte_end_ptr,
        )
    }
}

// Re-export functions for benchmarking when the 'bench' feature is enabled
#[cfg(feature = "bench")]
pub mod bench;

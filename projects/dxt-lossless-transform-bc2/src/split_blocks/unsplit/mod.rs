pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

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
    u32_detransform(input_ptr, output_ptr, len)
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
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_blocks_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_block_with_separate_pointers_x86(
    alphas_ptr: *const u64,
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        // disabled due to non-guaranteed performance on 32-bit
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::avx512_shuffle_with_components_intrinsics(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::avx2_shuffle_with_components(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_with_components(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        // disabled due to non-guaranteed performance on 32-bit
        if cfg!(target_feature = "avx512f") {
            avx512::avx512_shuffle_with_components_intrinsics(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::avx2_shuffle_with_components(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::shuffle_with_components(
                output_ptr,
                len,
                alphas_ptr as *const u8,
                colors_ptr as *const u8,
                indices_ptr as *const u8,
            );
            return;
        }
    }

    // Fallback to portable implementation
    u32_detransform_with_separate_pointers(alphas_ptr, colors_ptr, indices_ptr, output_ptr, len)
}

/// Unsplit BC2 blocks, putting them back into standard interleaved format from a separated alpha/color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - alphas_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/4 bytes
/// - indices_ptr must be valid for reads of len/4 bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that alphas_ptr, colors_ptr and indices_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_block_with_separate_pointers(
    alphas_ptr: *const u64,
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_block_with_separate_pointers_x86(
            alphas_ptr,
            colors_ptr,
            indices_ptr,
            output_ptr,
            len,
        )
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform_with_separate_pointers(alphas_ptr, colors_ptr, indices_ptr, output_ptr, len)
    }
}

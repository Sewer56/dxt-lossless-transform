mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;

#[cfg(feature = "bench")]
pub mod bench;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::permute_512_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::permd_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::unpck_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::permd_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::unpck_untransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_untransform(input_ptr, output_ptr, len)
}

/// Unsplit BC1 blocks, putting them back into standard interleaved format from a separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(8));

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_untransform(input_ptr, output_ptr, len)
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))] // API only used in non-x86 paths, but code preserved.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_with_separate_pointers_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        use dxt_lossless_transform_common::cpu_detect::*;
        if has_avx512f() {
            avx512::permute_512_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if has_avx2() {
            avx2::permd_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if has_sse2() {
            sse2::unpck_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::permd_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::unpck_untransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_untransform_with_separate_pointers(colors_ptr, indices_ptr, output_ptr, len)
}

/// Unsplit BC1 blocks, putting them back into standard interleaved format from a separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - colors_ptr must be valid for reads of len/2 bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that colors_ptr and indices_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))] // API only used in non-x86 paths, but code preserved.
pub(crate) unsafe fn untransform_with_separate_pointers(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_with_separate_pointers_x86(colors_ptr, indices_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_untransform_with_separate_pointers(colors_ptr, indices_ptr, output_ptr, len)
    }
}

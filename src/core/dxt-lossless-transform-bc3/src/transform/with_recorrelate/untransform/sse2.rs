#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;

use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::{
    recorrelate_ycocg_r_var1_sse2, recorrelate_ycocg_r_var2_sse2, recorrelate_ycocg_r_var3_sse2,
};

/// SSE2 implementation of BC3 untransform with YCoCg-R recorrelation.
///
/// # Safety
///
/// - alpha_endpoints_in must be valid for reads of num_blocks * 2 bytes
/// - alpha_indices_in must be valid for reads of num_blocks * 6 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - color_indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(
        alpha_endpoints_in,
        alpha_indices_in,
        colors_in,
        color_indices_in,
        output_ptr,
        num_blocks,
    )
}

unsafe fn untransform_recorr_var2(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(
        alpha_endpoints_in,
        alpha_indices_in,
        colors_in,
        color_indices_in,
        output_ptr,
        num_blocks,
    )
}

unsafe fn untransform_recorr_var3(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(
        alpha_endpoints_in,
        alpha_indices_in,
        colors_in,
        color_indices_in,
        output_ptr,
        num_blocks,
    )
}

/// SSE2 implementation of BC3 untransform with recorrelation.
///
/// Processes 4 blocks per iteration (64 bytes output).
/// BC3 block layout (16 bytes):
/// - bytes 0-1: alpha endpoints (2 bytes)
/// - bytes 2-7: alpha indices (6 bytes)
/// - bytes 8-11: color endpoints (4 bytes) - recorrelated
/// - bytes 12-15: color indices (4 bytes)
#[target_feature(enable = "sse2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alpha_endpoints_in: *const u16,
    mut alpha_indices_in: *const u16,
    mut colors_in: *const u32,
    mut color_indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Process 4 blocks at a time using SSE2 SIMD instructions
    let vectorized_blocks = num_blocks & !3; // Round down to multiple of 4
    let colors_end = colors_in.add(vectorized_blocks);

    // Cast alpha pointers to u64 for 8-byte reads
    let mut alpha_endpoints_ptr = alpha_endpoints_in as *const u64;
    let mut alpha_indices_ptr = alpha_indices_in as *const u64;

    // Main SIMD processing loop - handles 4 blocks per iteration
    while colors_in < colors_end {
        // Read alpha endpoints for 4 blocks (2 bytes * 4 = 8 bytes)
        let alpha_endpoints = alpha_endpoints_ptr.read_unaligned();
        alpha_endpoints_ptr = alpha_endpoints_ptr.add(1);

        // Write alpha endpoints for all 4 blocks
        write_u16(output_ptr, 0, shift_u64_u16(alpha_endpoints, 0));
        write_u16(output_ptr, 16, shift_u64_u16(alpha_endpoints, 16));
        write_u16(output_ptr, 32, shift_u64_u16(alpha_endpoints, 32));
        write_u16(output_ptr, 48, shift_u64_u16(alpha_endpoints, 48));

        // Handle alpha indices - read 24 bytes (6 bytes * 4 blocks) in 8-byte chunks
        let alpha_indices_0 = alpha_indices_ptr.read_unaligned();
        write_u16(output_ptr, 2, shift_u64_u16(alpha_indices_0, 0));
        write_u32(output_ptr, 4, shift_u64_u32(alpha_indices_0, 16)); // block 0 complete
        write_u16(output_ptr, 18, shift_u64_u16(alpha_indices_0, 48)); // block 1 start (2/6 bytes)

        let alpha_indices_1 = alpha_indices_ptr.add(1).read_unaligned();
        write_u32(output_ptr, 20, shift_u64_u32(alpha_indices_1, 0)); // block 1 complete (6/6 bytes)
        write_u32(output_ptr, 34, shift_u64_u32(alpha_indices_1, 32)); // block 2 start (4/6 bytes)

        let alpha_indices_2 = alpha_indices_ptr.add(2).read_unaligned();
        write_u16(output_ptr, 38, shift_u64_u16(alpha_indices_2, 0)); // block 2 complete (6/6 bytes)
        write_u64(output_ptr, 50, alpha_indices_2 >> 16); // block 3 (overwrites past end, fixed by color write)

        alpha_indices_ptr = alpha_indices_ptr.add(3);

        // Load colors for 4 blocks (4 bytes * 4 = 16 bytes)
        let colors = _mm_loadu_si128(colors_in as *const __m128i);
        colors_in = colors_in.add(4);

        // Apply recorrelation to the colors based on the variant
        let recorrelated_colors = match VARIANT {
            1 => recorrelate_ycocg_r_var1_sse2(colors),
            2 => recorrelate_ycocg_r_var2_sse2(colors),
            3 => recorrelate_ycocg_r_var3_sse2(colors),
            _ => unreachable_unchecked(),
        };

        // Load indices for 4 blocks
        let indices = _mm_loadu_si128(color_indices_in as *const __m128i);
        color_indices_in = color_indices_in.add(4);

        // Interleave colors and indices
        let low = _mm_unpacklo_epi32(recorrelated_colors, indices);
        let high = _mm_unpackhi_epi32(recorrelated_colors, indices);

        // Store interleaved colors+indices for all 4 blocks
        _mm_storel_epi64(output_ptr.add(8) as *mut __m128i, low);
        _mm_storeh_pd(output_ptr.add(24) as *mut f64, _mm_castsi128_pd(low));
        _mm_storel_epi64(output_ptr.add(40) as *mut __m128i, high);
        _mm_storeh_pd(output_ptr.add(56) as *mut f64, _mm_castsi128_pd(high));

        output_ptr = output_ptr.add(64);
    }

    // Update pointers to remaining position
    alpha_endpoints_in = alpha_endpoints_ptr as *const u16;
    alpha_indices_in = alpha_indices_ptr as *const u16;

    // Process remaining blocks using generic implementation
    let remaining_blocks = num_blocks - vectorized_blocks;
    if remaining_blocks > 0 {
        super::generic::untransform_with_recorrelate_generic(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            remaining_blocks,
            match VARIANT {
                1 => YCoCgVariant::Variant1,
                2 => YCoCgVariant::Variant2,
                3 => YCoCgVariant::Variant3,
                _ => unreachable_unchecked(),
            },
        );
    }
}

#[inline(always)]
unsafe fn write_u16(ptr: *mut u8, offset: usize, value: u16) {
    (ptr.add(offset) as *mut u16).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u32(ptr: *mut u8, offset: usize, value: u32) {
    (ptr.add(offset) as *mut u32).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u64(ptr: *mut u8, offset: usize, value: u64) {
    (ptr.add(offset) as *mut u64).write_unaligned(value);
}

#[inline(always)]
unsafe fn shift_u64_u16(value: u64, shift: usize) -> u16 {
    (value >> shift) as u16
}

#[inline(always)]
unsafe fn shift_u64_u32(value: u64, shift: usize) -> u32 {
    (value >> shift) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 8)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 8)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 8)]
    fn sse2_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_sse2() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "SSE2");
    }
}

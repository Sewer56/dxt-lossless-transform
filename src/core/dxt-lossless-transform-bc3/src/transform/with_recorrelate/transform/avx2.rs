use crate::transform::with_recorrelate::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx2::{
    decorrelate_ycocg_r_var1_avx2, decorrelate_ycocg_r_var2_avx2, decorrelate_ycocg_r_var3_avx2,
};

/// AVX2 implementation for BC3 transform with YCoCg-R decorrelation.
///
/// Processes 8 blocks (128 bytes) at a time using gather instructions for colors and indices,
/// and manual writes for alpha endpoints and indices.
#[target_feature(enable = "avx2")]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alpha_endpoints_out: *mut u16,
    mut alpha_indices_out: *mut u16,
    mut colors_out: *mut u32,
    mut color_indices_out: *mut u32,
    num_blocks: usize,
) {
    // Process 8 blocks (128 bytes) at a time
    let num_iterations = num_blocks / 8 * 8; // 8 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // 16 bytes per block

    // Create gather indices for colors (offset 8) and indices (offset 12)
    // For eight blocks, each 16 bytes apart
    let colour_offsets = _mm256_set_epi32(
        120, 104, 88, 72, 56, 40, 24, 8, // Block 8, 7, 6, 5, 4, 3, 2, 1 color offsets
    );
    let indices_offsets = _mm256_set_epi32(
        124, 108, 92, 76, 60, 44, 28, 12, // Block 8, 7, 6, 5, 4, 3, 2, 1 index offsets
    );

    // Create gather mask (all 1s)
    let mask = _mm256_set1_epi32(-1);

    while input_ptr < input_end {
        // Gather colors using _mm256_mask_i32gather_epi32 intrinsic
        let colours = _mm256_mask_i32gather_epi32::<1>(
            _mm256_setzero_si256(),  // src: source where no elements are gathered
            input_ptr as *const i32, // base_addr: base pointer
            colour_offsets,          // vindex: offsets to gather from
            mask,                    // mask: which elements to gather
        );

        // Gather indices using _mm256_mask_i32gather_epi32 intrinsic
        let indices = _mm256_mask_i32gather_epi32::<1>(
            _mm256_setzero_si256(),  // src: source where no elements are gathered
            input_ptr as *const i32, // base_addr: base pointer
            indices_offsets,         // vindex: offsets to gather from
            mask,                    // mask: which elements to gather
        );

        // Apply decorrelation to colors
        let colors_decorr = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colours),
            2 => decorrelate_ycocg_r_var2_avx2(colours),
            3 => decorrelate_ycocg_r_var3_avx2(colours),
            _ => unreachable_unchecked(),
        };

        // Write out all alpha endpoints (2 bytes each), for all 8 blocks
        // Combine 4 u16s into a u64 to reduce write operations
        write_alpha_endpoints_u64(alpha_endpoints_out as *mut u64, input_ptr, 0, 16, 32, 48);
        write_alpha_endpoints_u64(
            (alpha_endpoints_out as *mut u64).add(1),
            input_ptr,
            64,
            80,
            96,
            112,
        );
        alpha_endpoints_out = alpha_endpoints_out.add(8);

        // Write out all alpha indices components (6 bytes per block = 48 bytes total for 8 blocks)
        // Resolved and optimized out at compile time!
        let alpha_indices_out_ptr = alpha_indices_out as *mut u8;
        if cfg!(target_arch = "x86_64") {
            // Write out all alpha indices components 8 bytes at a time, for all 8 blocks
            write_alpha_bits_u64(alpha_indices_out_ptr, 0, input_ptr, 2);
            write_alpha_bits_u64(alpha_indices_out_ptr, 6, input_ptr, 18);
            write_alpha_bits_u64(alpha_indices_out_ptr, 12, input_ptr, 34);
            write_alpha_bits_u64(alpha_indices_out_ptr, 18, input_ptr, 50);
            write_alpha_bits_u64(alpha_indices_out_ptr, 24, input_ptr, 66);
            write_alpha_bits_u64(alpha_indices_out_ptr, 30, input_ptr, 82);
            write_alpha_bits_u64(alpha_indices_out_ptr, 36, input_ptr, 98);
            // Note: The u64 write overflows by 2 bytes; so on the last write, we need to not overflow, as to
            // not overwrite elements in the next section; so we do a regular write here.
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 42, 114);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 44, 116);
        } else {
            // Write out all alpha indices components (2 bytes then 4 bytes for each block), for all 8 blocks
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 0, 2);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 2, 4);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 6, 18);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 8, 20);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 12, 34);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 14, 36);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 18, 50);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 20, 52);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 24, 66);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 26, 68);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 30, 82);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 32, 84);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 36, 98);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 38, 100);
            write_alpha_bit_u16(alpha_indices_out_ptr, input_ptr, 42, 114);
            write_alpha_bit_u32(alpha_indices_out_ptr, input_ptr, 44, 116);
        }
        alpha_indices_out = alpha_indices_out.add(24); // 48 bytes = 24 u16s

        // Store results - each register now contains 8 blocks worth of data
        _mm256_storeu_si256(colors_out as *mut __m256i, colors_decorr);
        _mm256_storeu_si256(color_indices_out as *mut __m256i, indices);

        // Update pointers
        input_ptr = input_ptr.add(128); // Move forward 8 blocks
        colors_out = colors_out.add(8); // 8 u32s per m256i
        color_indices_out = color_indices_out.add(8); // 8 u32s per m256i
    }

    // Handle remaining blocks
    let remaining = num_blocks % 8;
    if remaining > 0 {
        let variant_enum = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::transform_with_decorrelate_generic(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            remaining,
            variant_enum,
        );
    }
}

#[inline(always)]
unsafe fn write_alpha_endpoints_u64(
    out_ptr: *mut u64,
    in_ptr: *const u8,
    offset0: usize,
    offset1: usize,
    offset2: usize,
    offset3: usize,
) {
    // Read 4 scattered u16 values
    let alpha_0 = (in_ptr.add(offset0) as *const u16).read_unaligned() as u64;
    let alpha_1 = (in_ptr.add(offset1) as *const u16).read_unaligned() as u64;
    let alpha_2 = (in_ptr.add(offset2) as *const u16).read_unaligned() as u64;
    let alpha_3 = (in_ptr.add(offset3) as *const u16).read_unaligned() as u64;

    // Combine into a single u64 via shifts and OR
    let combined = alpha_0 | (alpha_1 << 16) | (alpha_2 << 32) | (alpha_3 << 48);

    // Write as a single u64
    out_ptr.write_unaligned(combined);
}

#[inline(always)]
unsafe fn write_alpha_bit_u16(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u16)
        .write_unaligned((in_ptr.add(in_offset) as *const u16).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bit_u32(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u32)
        .write_unaligned((in_ptr.add(in_offset) as *const u32).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bits_u64(
    out_ptr: *mut u8,
    out_offset: usize,
    in_ptr: *const u8,
    in_offset: usize,
) {
    // Read both parts using unaligned loads
    let first_part = (in_ptr.add(in_offset) as *const u16).read_unaligned();
    let second_part = (in_ptr.add(in_offset + 2) as *const u32).read_unaligned();
    let combined_value = ((second_part as u64) << 16) | (first_part as u64);

    // Write using unaligned store
    (out_ptr.add(out_offset) as *mut u64).write_unaligned(combined_value);
}

// Wrappers for asm inspection and variant dispatch
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
#[inline]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
#[inline]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
#[inline]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<3>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

// Runtime dispatcher
#[inline(always)]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => transform_decorr_var1(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
        YCoCgVariant::Variant2 => transform_decorr_var2(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
        YCoCgVariant::Variant3 => transform_decorr_var3(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 16)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 16)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 16)]
    fn avx2_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx2() {
            return;
        }
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "AVX2");
    }
}

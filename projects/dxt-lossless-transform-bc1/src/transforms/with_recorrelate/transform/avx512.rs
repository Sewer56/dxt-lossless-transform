use crate::transforms::with_recorrelate::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx512::{
    decorrelate_ycocg_r_var1_avx512, decorrelate_ycocg_r_var2_avx512,
    decorrelate_ycocg_r_var3_avx512,
};

// Add local byte-index constants for AVX512 permutations
const PERM_COLORS_BYTES: [i8; 16] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30];
const PERM_INDICES_BYTES: [i8; 16] = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31];

#[target_feature(enable = "avx512f")]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut colours_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    // Load permutation patterns and sign-extend to dwords
    let perm_colors = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_COLORS_BYTES.as_ptr() as *const _));
    let perm_indices =
        _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_INDICES_BYTES.as_ptr() as *const _));

    // Pointer and block setup
    let blocks32 = num_blocks / 32;
    let input_end = input_ptr.add(blocks32 * 32 * 8); // blocks32 * 32 blocks per iteration * 8 bytes per block

    while input_ptr < input_end {
        // Load 256 bytes (4 × 64-byte ZMM registers) = 32 BC1 blocks.
        let in0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let in1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        let in2 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
        let in3 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);
        input_ptr = input_ptr.add(256);

        // Split and decorrelate colors
        let col0 = _mm512_permutex2var_epi32(in0, perm_colors, in1);
        let col0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx512(col0),
            2 => decorrelate_ycocg_r_var2_avx512(col0),
            3 => decorrelate_ycocg_r_var3_avx512(col0),
            _ => unreachable_unchecked(),
        };
        _mm512_storeu_si512(colours_out as *mut __m512i, col0);

        let col1 = _mm512_permutex2var_epi32(in2, perm_colors, in3);
        let col1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx512(col1),
            2 => decorrelate_ycocg_r_var2_avx512(col1),
            3 => decorrelate_ycocg_r_var3_avx512(col1),
            _ => unreachable_unchecked(),
        };
        _mm512_storeu_si512(colours_out.add(16) as *mut __m512i, col1);
        colours_out = colours_out.add(32);

        // Split indices and store
        let idx0 = _mm512_permutex2var_epi32(in0, perm_indices, in1);
        let idx1 = _mm512_permutex2var_epi32(in2, perm_indices, in3);
        _mm512_storeu_si512(indices_out as *mut __m512i, idx0);
        _mm512_storeu_si512(indices_out.add(16) as *mut __m512i, idx1);
        indices_out = indices_out.add(32);
    }

    let remaining_blocks = num_blocks % 32;
    let variant_enum = match VARIANT {
        1 => YCoCgVariant::Variant1,
        2 => YCoCgVariant::Variant2,
        3 => YCoCgVariant::Variant3,
        _ => unreachable_unchecked(),
    };
    generic::transform_with_decorrelate_generic(
        input_ptr,
        colours_out,
        indices_out,
        remaining_blocks,
        variant_enum,
    );
}

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<3>(input_ptr, colours_out, indices_out, num_blocks)
}

// Runtime dispatcher
#[inline(always)]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => {
            transform_decorr_var1(input_ptr, colours_out, indices_out, num_blocks)
        }
        YCoCgVariant::Variant2 => {
            transform_decorr_var2(input_ptr, colours_out, indices_out, num_blocks)
        }
        YCoCgVariant::Variant3 => {
            transform_decorr_var3(input_ptr, colours_out, indices_out, num_blocks)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 64)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 64)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 64)]
    fn avx512_transform_roundtrip(
        #[case] func: unsafe fn(*const u8, *mut u32, *mut u32, usize),
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512f() {
            return;
        }

        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "AVX512");
    }
}

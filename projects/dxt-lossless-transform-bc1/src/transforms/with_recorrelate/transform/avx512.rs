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
    input_ptr: *const u8,
    colours_ptr: *mut u32,
    indices_ptr: *mut u32,
    num_blocks: usize,
) {
    // Pointer and block setup
    let mut src = input_ptr;
    let mut col_out = colours_ptr;
    let mut idx_out = indices_ptr;
    let blocks32 = num_blocks / 32;
    let rem = num_blocks % 32;
    if blocks32 > 0 {
        // Load permutation patterns and sign-extend to dwords
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_INDICES_BYTES.as_ptr() as *const _));
        let input_end = input_ptr.add(blocks32 * 32 * 8); // blocks32 * 32 blocks per iteration * 8 bytes per block
        while src < input_end {
            // Load 256 bytes = 32 blocks
            let src0 = _mm512_loadu_si512(src as *const __m512i);
            let src1 = _mm512_loadu_si512(src.add(64) as *const __m512i);
            let src2 = _mm512_loadu_si512(src.add(128) as *const __m512i);
            let src3 = _mm512_loadu_si512(src.add(192) as *const __m512i);
            src = src.add(256);
            // Split and decorrelate colors
            let col0 = _mm512_permutex2var_epi32(src0, perm_colors, src1);
            let rec0 = match VARIANT {
                1 => decorrelate_ycocg_r_var1_avx512(col0),
                2 => decorrelate_ycocg_r_var2_avx512(col0),
                3 => decorrelate_ycocg_r_var3_avx512(col0),
                _ => unreachable_unchecked(),
            };
            _mm512_storeu_si512(col_out as *mut __m512i, rec0);
            let col1 = _mm512_permutex2var_epi32(src2, perm_colors, src3);
            let rec1 = match VARIANT {
                1 => decorrelate_ycocg_r_var1_avx512(col1),
                2 => decorrelate_ycocg_r_var2_avx512(col1),
                3 => decorrelate_ycocg_r_var3_avx512(col1),
                _ => unreachable_unchecked(),
            };
            _mm512_storeu_si512(col_out.add(16) as *mut __m512i, rec1);
            col_out = col_out.add(32);
            // Split indices and store
            let idx0 = _mm512_permutex2var_epi32(src0, perm_indices, src1);
            _mm512_storeu_si512(idx_out as *mut __m512i, idx0);
            let idx1 = _mm512_permutex2var_epi32(src2, perm_indices, src3);
            _mm512_storeu_si512(idx_out.add(16) as *mut __m512i, idx1);
            idx_out = idx_out.add(32);
        }
    }
    if rem > 0 {
        let variant_enum = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::transform_with_decorrelate_generic(src, col_out, idx_out, rem, variant_enum);
    }
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
    colours_ptr: *mut u32,
    indices_ptr: *mut u32,
    num_blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => {
            transform_decorr_var1(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::Variant2 => {
            transform_decorr_var2(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::Variant3 => {
            transform_decorr_var3(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_recorrelate::untransform::untransform_with_recorrelate;

    #[rstest]
    #[case(transform_decorr_var1, YCoCgVariant::Variant1)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3)]
    fn avx512_transform_roundtrip(
        #[case] func: unsafe fn(*const u8, *mut u32, *mut u32, usize),
        #[case] variant: YCoCgVariant,
    ) {
        for num_blocks in 1..=128 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut transformed = vec![0u8; len];
            let mut reconstructed = vec![0u8; len];
            unsafe {
                func(
                    input.as_ptr(),
                    transformed.as_mut_ptr() as *mut u32,
                    transformed.as_mut_ptr().add(len / 2) as *mut u32,
                    num_blocks,
                );
                untransform_with_recorrelate(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    num_blocks * 8,
                    variant,
                );
            }
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "Mismatch AVX512 roundtrip variant {variant:?}",
            );
        }
    }
}

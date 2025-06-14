use crate::transforms::with_recorrelate::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx2::{
    decorrelate_ycocg_r_var1_avx2, decorrelate_ycocg_r_var2_avx2, decorrelate_ycocg_r_var3_avx2,
};

/// AVX2 implementation for transform with YCoCg-R decorrelation.
#[target_feature(enable = "avx2")]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut colours_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    let blocks16 = num_blocks / 16;
    let input_end = input_ptr.add(blocks16 * 16 * 8); // blocks16 * 16 blocks per iteration * 8 bytes per block
    while input_ptr < input_end {
        // Load 16 blocks = 128 bytes
        let data0 = _mm256_loadu_si256(input_ptr as *const __m256i);
        let data1 = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i);
        let data2 = _mm256_loadu_si256(input_ptr.add(64) as *const __m256i);
        let data3 = _mm256_loadu_si256(input_ptr.add(96) as *const __m256i);
        input_ptr = input_ptr.add(128);

        // Split colours and indices using shuffle patterns per 128-bit lane
        let colors_only_0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data0),
            _mm256_castsi256_ps(data1),
            0x88,
        ));
        let colors0 = _mm256_permute4x64_epi64(colors_only_0, 0xD8); // 0b11011000 = 216

        let colors_only_1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data3),
            0x88,
        ));
        let colors1 = _mm256_permute4x64_epi64(colors_only_1, 0xD8); // 0b11011000 = 216

        let indices_only_0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data0),
            _mm256_castsi256_ps(data1),
            0xDD,
        ));
        let indices0 = _mm256_permute4x64_epi64(indices_only_0, 0xD8); // 0b11011000 = 216

        let indices_only_1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data3),
            0xDD,
        ));
        let indices1 = _mm256_permute4x64_epi64(indices_only_1, 0xD8); // 0b11011000 = 216

        // Apply decorrelation
        let colors0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors0),
            2 => decorrelate_ycocg_r_var2_avx2(colors0),
            3 => decorrelate_ycocg_r_var3_avx2(colors0),
            _ => unreachable_unchecked(),
        };
        let colors1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors1),
            2 => decorrelate_ycocg_r_var2_avx2(colors1),
            3 => decorrelate_ycocg_r_var3_avx2(colors1),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm256_storeu_si256(colours_out as *mut __m256i, colors0);
        _mm256_storeu_si256(colours_out.add(8) as *mut __m256i, colors1);
        _mm256_storeu_si256(indices_out as *mut __m256i, indices0);
        _mm256_storeu_si256(indices_out.add(8) as *mut __m256i, indices1);

        colours_out = colours_out.add(16);
        indices_out = indices_out.add(16);
    }
    let remaining = num_blocks % 16;
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
        remaining,
        variant_enum,
    );
}

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
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
    #[case(transform_decorr_var1, YCoCgVariant::Variant1)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3)]
    fn avx2_transform_roundtrip(
        #[case] func: unsafe fn(*const u8, *mut u32, *mut u32, usize),
        #[case] variant: YCoCgVariant,
    ) {
        run_with_decorrelate_transform_roundtrip_test(func, variant, 128, "AVX2");
    }
}

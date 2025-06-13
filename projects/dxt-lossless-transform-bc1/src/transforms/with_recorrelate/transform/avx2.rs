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
    input_ptr: *const u8,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    num_blocks: usize,
) {
    let mut in_ptr = input_ptr;
    let blocks16 = num_blocks / 16;
    for _ in 0..blocks16 {
        // Load 16 blocks = 128 bytes
        let data0 = _mm256_loadu_si256(in_ptr as *const __m256i);
        let data1 = _mm256_loadu_si256(in_ptr.add(32) as *const __m256i);
        let data2 = _mm256_loadu_si256(in_ptr.add(64) as *const __m256i);
        let data3 = _mm256_loadu_si256(in_ptr.add(96) as *const __m256i);
        in_ptr = in_ptr.add(128);

        // Split colours and indices using shuffle patterns per 128-bit lane
        let col0_shuffled = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data0),
            _mm256_castsi256_ps(data1),
            0x88,
        ));
        let col0 = _mm256_permute4x64_epi64(col0_shuffled, 0xD8); // 0b11011000 = 216
        let col1_shuffled = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data3),
            0x88,
        ));
        let col1 = _mm256_permute4x64_epi64(col1_shuffled, 0xD8); // 0b11011000 = 216
        let idx0_shuffled = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data0),
            _mm256_castsi256_ps(data1),
            0xDD,
        ));
        let idx0 = _mm256_permute4x64_epi64(idx0_shuffled, 0xD8); // 0b11011000 = 216
        let idx1_shuffled = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data3),
            0xDD,
        ));
        let idx1 = _mm256_permute4x64_epi64(idx1_shuffled, 0xD8); // 0b11011000 = 216

        // Apply decorrelation
        let rec_lo = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(col0),
            2 => decorrelate_ycocg_r_var2_avx2(col0),
            3 => decorrelate_ycocg_r_var3_avx2(col0),
            _ => unreachable_unchecked(),
        };
        let rec_hi = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(col1),
            2 => decorrelate_ycocg_r_var2_avx2(col1),
            3 => decorrelate_ycocg_r_var3_avx2(col1),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm256_storeu_si256(colours_ptr as *mut __m256i, rec_lo);
        _mm256_storeu_si256(colours_ptr.add(8) as *mut __m256i, rec_hi);
        _mm256_storeu_si256(indices_ptr as *mut __m256i, idx0);
        _mm256_storeu_si256(indices_ptr.add(8) as *mut __m256i, idx1);

        colours_ptr = colours_ptr.add(16);
        indices_ptr = indices_ptr.add(16);
    }
    let rem = num_blocks % 16;
    if rem > 0 {
        let variant_enum = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::transform_with_decorrelate_generic(
            in_ptr,
            colours_ptr,
            indices_ptr,
            rem,
            variant_enum,
        );
    }
}

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<3>(input_ptr, colours_out, indices_out, num_blocks)
}

// Runtime dispatcher
#[allow(dead_code)]
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
    fn avx2_transform_roundtrip(
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
                "Mismatch AVX2 roundtrip variant {variant:?}",
            );
        }
    }
}

use crate::transforms::with_recorrelate::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::sse2::{
    decorrelate_ycocg_r_var1_sse2, decorrelate_ycocg_r_var2_sse2, decorrelate_ycocg_r_var3_sse2,
};

// Const-generic worker
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
unsafe fn transform_decorr<const VARIANT: u8>(
    input_ptr: *const u8,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    num_blocks: usize,
) {
    let mut in_ptr = input_ptr;
    let blocks8 = num_blocks / 8;
    let input_end = input_ptr.add(blocks8 * 8 * 8); // blocks8 * 8 blocks per iteration * 8 bytes per block
    while in_ptr < input_end {
        // Load four 16-byte chunks = 8 blocks
        let data0 = _mm_loadu_si128(in_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(in_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(in_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(in_ptr.add(48) as *const __m128i);
        in_ptr = in_ptr.add(64);

        // Split colors and indices using shufps patterns
        let col0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0x88,
        ));
        let col1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0x88,
        ));
        let idx0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0xDD,
        ));
        let idx1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0xDD,
        ));

        // Apply decorrelation
        let rec_lo = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(col0),
            2 => decorrelate_ycocg_r_var2_sse2(col0),
            3 => decorrelate_ycocg_r_var3_sse2(col0),
            _ => unreachable_unchecked(),
        };
        let rec_hi = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(col1),
            2 => decorrelate_ycocg_r_var2_sse2(col1),
            3 => decorrelate_ycocg_r_var3_sse2(col1),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm_storeu_si128(colours_ptr as *mut __m128i, rec_lo);
        _mm_storeu_si128((colours_ptr as *mut __m128i).add(1), rec_hi);
        _mm_storeu_si128(indices_ptr as *mut __m128i, idx0);
        _mm_storeu_si128((indices_ptr as *mut __m128i).add(1), idx1);

        colours_ptr = colours_ptr.add(8); // 32 bytes
        indices_ptr = indices_ptr.add(8); // 32 bytes
    }
    // Handle any remaining blocks
    let rem = num_blocks % 8;
    if rem > 0 {
        // Map const generic variant to enum for generic fallback
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
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    colours_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(input_ptr, colours_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
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
    fn sse2_transform_roundtrip(
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
                "Mismatch SSE2 roundtrip variant {variant:?}"
            );
        }
    }
}

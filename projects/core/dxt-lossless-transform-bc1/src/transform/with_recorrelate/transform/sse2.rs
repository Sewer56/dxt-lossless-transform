use crate::transform::with_recorrelate::transform::generic;
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
    mut input_ptr: *const u8,
    mut colours_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    let blocks8 = num_blocks / 8; // round down via division
    let input_end = input_ptr.add(blocks8 * 8 * 8); // blocks8 * 8 blocks per iteration * 8 bytes per block
    while input_ptr < input_end {
        // Load four 16-byte chunks = 8 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        input_ptr = input_ptr.add(64);

        // Split colors and indices using shufps patterns
        let colors0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0x88,
        ));
        let colors1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0x88,
        ));
        let indices0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0xDD,
        ));
        let indices1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0xDD,
        ));

        // Apply decorrelation
        let colors0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colors0),
            2 => decorrelate_ycocg_r_var2_sse2(colors0),
            3 => decorrelate_ycocg_r_var3_sse2(colors0),
            _ => unreachable_unchecked(),
        };
        let colors1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colors1),
            2 => decorrelate_ycocg_r_var2_sse2(colors1),
            3 => decorrelate_ycocg_r_var3_sse2(colors1),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm_storeu_si128(colours_out as *mut __m128i, colors0);
        _mm_storeu_si128((colours_out as *mut __m128i).add(1), colors1);
        _mm_storeu_si128(indices_out as *mut __m128i, indices0);
        _mm_storeu_si128((indices_out as *mut __m128i).add(1), indices1);

        colours_out = colours_out.add(8); // 32 bytes
        indices_out = indices_out.add(8); // 32 bytes
    }
    // Handle any remaining blocks
    let remaining_blocks = num_blocks % 8;
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
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
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
#[inline]
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
#[inline]
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
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 16)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 16)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 16)]
    fn sse2_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "SSE2");
    }
}

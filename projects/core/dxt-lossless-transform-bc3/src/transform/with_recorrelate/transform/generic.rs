use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic implementation of BC3 transform with YCoCg-R decorrelation.
///
/// Splits standard interleaved BC3 blocks into separate alpha endpoints, alpha indices,
/// color, and color index buffers, applying YCoCg-R decorrelation to color endpoints.
///
/// # Safety
///
/// - input_ptr must be valid for reads of num_blocks * 16 bytes
/// - alpha_endpoints_out must be valid for writes of num_blocks * 2 bytes
/// - alpha_indices_out must be valid for writes of num_blocks * 6 bytes
/// - colors_out must be valid for writes of num_blocks * 4 bytes
/// - color_indices_out must be valid for writes of num_blocks * 4 bytes
/// - decorrelation_mode must be a valid [`YCoCgVariant`]
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn transform_with_decorrelate_generic(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
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

#[allow(dead_code)]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alpha_endpoints_out: *mut u16,
    mut alpha_indices_out: *mut u16,
    mut colors_out: *mut u32,
    mut color_indices_out: *mut u32,
    num_blocks: usize,
) {
    let input_end = input_ptr.add(num_blocks * 16);
    while input_ptr < input_end {
        // Read BC3 block (16 bytes)
        // Offset 0-1: Alpha endpoints (2 bytes)
        let alpha_endpoints = input_ptr.read_u16_at(0);

        // Offset 2-7: Alpha indices (6 bytes)
        let alpha_indices_1 = input_ptr.read_u16_at(2);
        let alpha_indices_2 = input_ptr.read_u32_at(4);

        // Offset 8-11: Color endpoints (4 bytes)
        let color_raw = input_ptr.read_u32_at(8);

        // Offset 12-15: Color indices (4 bytes)
        let color_indices = input_ptr.read_u32_at(12);

        input_ptr = input_ptr.add(16);

        // Extract two 16-bit colors
        let color0 = Color565::from_raw(color_raw as u16);
        let color1 = Color565::from_raw((color_raw >> 16) as u16);

        // Apply YCoCg-R decorrelation based on variant
        let (decorr0, decorr1) = match VARIANT {
            1 => (
                color0.decorrelate_ycocg_r_var1(),
                color1.decorrelate_ycocg_r_var1(),
            ),
            2 => (
                color0.decorrelate_ycocg_r_var2(),
                color1.decorrelate_ycocg_r_var2(),
            ),
            3 => (
                color0.decorrelate_ycocg_r_var3(),
                color1.decorrelate_ycocg_r_var3(),
            ),
            _ => unreachable_unchecked(),
        };

        // Pack decorrelated colors into u32
        let decorrelated_colors =
            (decorr0.raw_value() as u32) | ((decorr1.raw_value() as u32) << 16);

        // Write to separate buffers
        alpha_endpoints_out.write_u16_at(0, alpha_endpoints);
        alpha_indices_out.write_u16_at(0, alpha_indices_1);
        alpha_indices_out.write_u32_at(2, alpha_indices_2);
        colors_out.write_u32_at(0, decorrelated_colors);
        color_indices_out.write_u32_at(0, color_indices);

        alpha_endpoints_out = alpha_endpoints_out.add(1);
        alpha_indices_out = alpha_indices_out.add(3); // 6 bytes = 3 u16
        colors_out = colors_out.add(1);
        color_indices_out = color_indices_out.add(1);
    }
}

// Wrapper functions for testing with specific variants
#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 2)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 2)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 2)]
    fn roundtrip_transform_with_decorrelate(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "generic");
    }
}

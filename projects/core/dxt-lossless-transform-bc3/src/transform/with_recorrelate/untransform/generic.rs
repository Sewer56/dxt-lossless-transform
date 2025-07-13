use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic implementation of BC3 untransform with YCoCg-R recorrelation.
///
/// Combines separate alpha endpoints, alpha indices, color, and color index buffers back
/// into standard interleaved BC3 blocks, applying YCoCg-R recorrelation to color endpoints.
///
/// # Safety
///
/// - alpha_endpoints_in must be valid for reads of num_blocks * 2 bytes
/// - alpha_indices_in must be valid for reads of num_blocks * 6 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - color_indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate_generic(
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
#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alpha_endpoints_in: *const u16,
    mut alpha_indices_in: *const u16,
    mut colors_in: *const u32,
    mut color_indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    let alpha_endpoints_end = alpha_endpoints_in.add(num_blocks);
    while alpha_endpoints_in < alpha_endpoints_end {
        // Read alpha endpoints, alpha indices, colors, and color indices
        let alpha_endpoints = alpha_endpoints_in.read_u16_at(0);
        let alpha_indices_1 = alpha_indices_in.read_u16_at(0);
        let alpha_indices_2 = alpha_indices_in.read_u32_at(2);
        let color_raw = colors_in.read_u32_at(0);
        let color_indices = color_indices_in.read_u32_at(0);

        alpha_endpoints_in = alpha_endpoints_in.add(1);
        alpha_indices_in = alpha_indices_in.add(3); // 6 bytes = 3 u16
        colors_in = colors_in.add(1);
        color_indices_in = color_indices_in.add(1);

        // Extract both [`Color565`] values from the u32
        let color0 = Color565::from_raw(color_raw as u16);
        let color1 = Color565::from_raw((color_raw >> 16) as u16);

        // Apply recorrelation to both colors based on the variant
        let (recorr_color0, recorr_color1) = match VARIANT {
            1 => (
                color0.recorrelate_ycocg_r_var1(),
                color1.recorrelate_ycocg_r_var1(),
            ),
            2 => (
                color0.recorrelate_ycocg_r_var2(),
                color1.recorrelate_ycocg_r_var2(),
            ),
            3 => (
                color0.recorrelate_ycocg_r_var3(),
                color1.recorrelate_ycocg_r_var3(),
            ),
            _ => unreachable_unchecked(),
        };

        // Pack both recorrelated colors back into u32
        let recorrelated_colors =
            (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

        // Write BC3 block: alpha endpoints (2 bytes) + alpha indices (6 bytes) + colors (4 bytes) + color indices (4 bytes)
        output_ptr.write_u16_at(0, alpha_endpoints);
        output_ptr.write_u16_at(2, alpha_indices_1);
        output_ptr.write_u32_at(4, alpha_indices_2);
        output_ptr.write_u32_at(8, recorrelated_colors);
        output_ptr.write_u32_at(12, color_indices);

        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 2)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 2)]
    fn roundtrip_untransform_with_recorrelate(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "generic");
    }
}

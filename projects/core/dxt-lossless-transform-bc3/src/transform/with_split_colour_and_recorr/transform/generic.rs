use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-colour and decorrelate transform for BC3.
/// Splits standard interleaved BC3 blocks into separate arrays of alpha_endpoints, alpha_indices,
/// decorrelated_color0, decorrelated_color1 and color_indices while applying YCoCg decorrelation
/// to the color endpoints.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_endpoints_out` must be valid for writes of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `decorrelated_color0_out` must be valid for writes of `block_count * 2` bytes
/// - `decorrelated_color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - `decorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    decorrelated_color0_out: *mut u16,
    decorrelated_color1_out: *mut u16,
    color_indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    // Initialize pointers
    let mut input_ptr = input_ptr;
    let mut alpha_endpoints_ptr = alpha_endpoints_out;
    let mut alpha_indices_ptr = alpha_indices_out;
    let mut decorrelated_color0_ptr = decorrelated_color0_out;
    let mut decorrelated_color1_ptr = decorrelated_color1_out;
    let mut color_indices_ptr = color_indices_out;

    // Process each block
    let input_end = input_ptr.add(block_count * 16);
    while input_ptr < input_end {
        // Read BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]
        let alpha_endpoints = input_ptr.read_u16_at(0);

        // Read alpha indices (6 bytes) through three u16 reads
        let alpha_idx0 = input_ptr.read_u16_at(2);
        let alpha_idx1 = input_ptr.read_u16_at(4);
        let alpha_idx2 = input_ptr.read_u16_at(6);

        let color0_raw = input_ptr.read_u16_at(8);
        let color1_raw = input_ptr.read_u16_at(10);
        let color_indices = input_ptr.read_u32_at(12);

        // Apply YCoCg decorrelation to colors
        let color0 = Color565::from_raw(color0_raw);
        let color1 = Color565::from_raw(color1_raw);

        let decorr_color0 = match decorrelation_mode {
            YCoCgVariant::Variant1 => color0.decorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => color0.decorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => color0.decorrelate_ycocg_r_var3(),
            YCoCgVariant::None => unreachable_unchecked(),
        };

        let decorr_color1 = match decorrelation_mode {
            YCoCgVariant::Variant1 => color1.decorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => color1.decorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => color1.decorrelate_ycocg_r_var3(),
            YCoCgVariant::None => unreachable_unchecked(),
        };

        // Write to separate arrays
        alpha_endpoints_ptr.write_u16_at(0, alpha_endpoints);

        // Write alpha indices (6 bytes) through three u16 writes
        alpha_indices_ptr.write_u16_at(0, alpha_idx0);
        alpha_indices_ptr.write_u16_at(2, alpha_idx1);
        alpha_indices_ptr.write_u16_at(4, alpha_idx2);

        decorrelated_color0_ptr.write_u16_at(0, decorr_color0.raw_value());
        decorrelated_color1_ptr.write_u16_at(0, decorr_color1.raw_value());
        color_indices_ptr.write_u32_at(0, color_indices);

        // Advance all pointers
        input_ptr = input_ptr.add(16);
        alpha_endpoints_ptr = alpha_endpoints_ptr.add(1);
        alpha_indices_ptr = alpha_indices_ptr.add(3); // 6 bytes = 3 u16
        decorrelated_color0_ptr = decorrelated_color0_ptr.add(1);
        decorrelated_color1_ptr = decorrelated_color1_ptr.add(1);
        color_indices_ptr = color_indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn generic_transform_roundtrip() {
        // Generic processes 16 bytes per iteration (* 2 / 16 == 2)
        run_split_colour_and_recorr_transform_roundtrip_test(transform_with_split_colour_and_recorr, 2, "Generic");
    }
}

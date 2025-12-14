use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

/// Generic fallback implementation of split-colour and decorrelate transform for BC2.
/// Splits standard interleaved BC2 blocks into separate arrays of alpha, colour0, colour1 and indices
/// while applying YCoCg decorrelation to the color endpoints.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_ptr` must be valid for writes of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
/// - `decorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    // Initialize pointers
    let mut input_ptr = input_ptr;
    let mut alpha_ptr = alpha_out;
    let mut color0_ptr = color0_out;
    let mut color1_ptr = color1_out;
    let mut indices_ptr = indices_out;

    // Process each block
    let input_end = input_ptr.add(block_count * 16);
    while input_ptr < input_end {
        // Read BC2 block format: [alpha: u64, color0: u16, color1: u16, indices: u32]
        let alpha = (input_ptr as *const u64).read_unaligned();
        let color0_raw = (input_ptr.add(8) as *const u16).read_unaligned();
        let color1_raw = (input_ptr.add(10) as *const u16).read_unaligned();
        let indices = (input_ptr.add(12) as *const u32).read_unaligned();

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
        alpha_ptr.write_unaligned(alpha);
        color0_ptr.write_unaligned(decorr_color0.raw_value());
        color1_ptr.write_unaligned(decorr_color1.raw_value());
        indices_ptr.write_unaligned(indices);

        // Advance all pointers
        input_ptr = input_ptr.add(16);
        alpha_ptr = alpha_ptr.add(1);
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn generic_transform_roundtrip(#[case] variant: YCoCgVariant) {
        // Generic processes 16 bytes per iteration (* 2 / 16 == 2)
        run_split_colour_and_recorr_transform_roundtrip_test(
            transform_with_split_colour_and_recorr,
            variant,
            2,
            "Generic",
        );
    }
}

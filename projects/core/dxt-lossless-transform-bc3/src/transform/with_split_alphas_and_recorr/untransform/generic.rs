use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-alphas and recorrelate untransform for BC3.
/// Combines separate arrays of alpha0, alpha1, alpha_indices, decorrelated_colors and
/// color_indices back into standard interleaved BC3 blocks while applying YCoCg recorrelation
/// to the color endpoints.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `decorrelated_colors_out` must be valid for reads of `block_count * 4` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn untransform_with_split_alphas_and_recorr(
    mut alpha0_out: *const u8,
    mut alpha1_out: *const u8,
    mut alpha_indices_out: *const u16,
    mut decorrelated_colors_out: *const u32,
    mut color_indices_out: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    // Process each block
    let alpha0_end = alpha0_out.add(block_count);
    while alpha0_out < alpha0_end {
        // Read from separate arrays
        let alpha0 = alpha0_out.read();
        let alpha1 = alpha1_out.read();

        // Read alpha indices (6 bytes) through three u16 reads
        let alpha_idx0 = alpha_indices_out.read_u16_at(0);
        let alpha_idx1 = alpha_indices_out.read_u16_at(2);
        let alpha_idx2 = alpha_indices_out.read_u16_at(4);

        let packed_colors = decorrelated_colors_out.read_u32_at(0);
        let color_indices = color_indices_out.read_u32_at(0);

        // Unpack decorrelated colors
        let decorr_color0_raw = (packed_colors & 0xFFFF) as u16;
        let decorr_color1_raw = (packed_colors >> 16) as u16;

        // Apply YCoCg recorrelation to colors
        let decorr_color0 = Color565::from_raw(decorr_color0_raw);
        let decorr_color1 = Color565::from_raw(decorr_color1_raw);

        let color0 = match recorrelation_mode {
            YCoCgVariant::Variant1 => decorr_color0.recorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => decorr_color0.recorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => decorr_color0.recorrelate_ycocg_r_var3(),
            YCoCgVariant::None => unreachable_unchecked(),
        };

        let color1 = match recorrelation_mode {
            YCoCgVariant::Variant1 => decorr_color1.recorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => decorr_color1.recorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => decorr_color1.recorrelate_ycocg_r_var3(),
            YCoCgVariant::None => unreachable_unchecked(),
        };

        // Write BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]
        output_ptr.write(alpha0);
        output_ptr.add(1).write(alpha1);

        // Write alpha indices (6 bytes) through three u16 writes
        output_ptr.write_u16_at(2, alpha_idx0);
        output_ptr.write_u16_at(4, alpha_idx1);
        output_ptr.write_u16_at(6, alpha_idx2);

        output_ptr.write_u16_at(8, color0.raw_value());
        output_ptr.write_u16_at(10, color1.raw_value());
        output_ptr.write_u32_at(12, color_indices);

        // Advance all pointers
        alpha0_out = alpha0_out.add(1);
        alpha1_out = alpha1_out.add(1);
        alpha_indices_out = alpha_indices_out.add(3); // 6 bytes = 3 u16
        decorrelated_colors_out = decorrelated_colors_out.add(1);
        color_indices_out = color_indices_out.add(1);
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[test]
    fn can_untransform_unaligned() {
        // 1 block processed per iteration (no SIMD) (* 2 == 2)
        run_with_split_alphas_and_recorr_untransform_unaligned_test(
            untransform_with_split_alphas_and_recorr,
            2,
            "untransform_with_split_alphas_and_recorr (generic, unaligned)",
        );
    }
}

use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

/// Generic fallback implementation of split-colour and recorrelate untransform for BC2.
/// Combines separate arrays of alpha, colour0, colour1 and indices back into standard interleaved BC2 blocks
/// while applying YCoCg recorrelation to the color endpoints.
///
/// # Safety
///
/// - `alpha_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for reads of `block_count * 2` bytes
/// - `color1_ptr` must be valid for reads of `block_count * 2` bytes
/// - `indices_ptr` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[inline]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_split_colour_recorr_var1(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_split_colour_recorr_var2(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_split_colour_recorr_var3(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_split_colour_recorr_var1(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<1>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var2(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<2>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var3(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<3>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

/// Generic fallback implementation of split-colour and recorrelate untransform for BC2.
/// Combines separate arrays of alpha, colour0, colour1 and indices back into standard interleaved BC2 blocks
/// while applying YCoCg recorrelation to the color endpoints.
///
/// # Safety
///
/// - `alpha_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for reads of `block_count * 2` bytes
/// - `color1_ptr` must be valid for reads of `block_count * 2` bytes
/// - `indices_ptr` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[inline]
unsafe fn untransform_split_colour_recorr<const VARIANT: u8>(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Calculate end pointer for alpha (as it's the largest element)
    let alpha_ptr_end = alpha_ptr.add(block_count);

    while alpha_ptr < alpha_ptr_end {
        // Read the split values
        let alpha = alpha_ptr.read_unaligned();
        let color0_raw = color0_ptr.read_unaligned();
        let color1_raw = color1_ptr.read_unaligned();
        let indices = indices_ptr.read_unaligned();

        // Apply YCoCg recorrelation to colors
        let color0 = Color565::from_raw(color0_raw);
        let color1 = Color565::from_raw(color1_raw);

        let recorr_color0 = match VARIANT {
            1 => color0.recorrelate_ycocg_r_var1(),
            2 => color0.recorrelate_ycocg_r_var2(),
            3 => color0.recorrelate_ycocg_r_var3(),
            _ => unreachable_unchecked(),
        };

        let recorr_color1 = match VARIANT {
            1 => color1.recorrelate_ycocg_r_var1(),
            2 => color1.recorrelate_ycocg_r_var2(),
            3 => color1.recorrelate_ycocg_r_var3(),
            _ => unreachable_unchecked(),
        };

        // Write BC2 block format: [alpha: u64, color0: u16, color1: u16, indices: u32]
        (output_ptr as *mut u64).write_unaligned(alpha);
        (output_ptr.add(8) as *mut u16).write_unaligned(recorr_color0.raw_value());
        (output_ptr.add(10) as *mut u16).write_unaligned(recorr_color1.raw_value());
        (output_ptr.add(12) as *mut u32).write_unaligned(indices);

        // Advance all pointers
        alpha_ptr = alpha_ptr.add(1);
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
        output_ptr = output_ptr.add(16);
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
    fn generic_untransform_roundtrip(#[case] variant: YCoCgVariant) {
        // Generic processes 16 bytes per iteration (* 2 / 16 == 2)
        run_split_colour_and_recorr_untransform_roundtrip_test(
            untransform_with_split_colour_and_recorr,
            variant,
            2,
            "Generic",
        );
    }
}

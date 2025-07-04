use core::hint::unreachable_unchecked;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

/// Generic implementation of BC2 untransform with YCoCg-R recorrelation.
///
/// Combines separate alpha, color, and index buffers back into standard interleaved BC2 blocks,
/// applying YCoCg-R recorrelation to color endpoints.
///
/// # Safety
///
/// - alphas_in must be valid for reads of num_blocks * 8 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate_generic(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alphas_in: *const u64,
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    let alphas_end = alphas_in.add(num_blocks);
    while alphas_in < alphas_end {
        // Read alpha, colors, and indices
        let alpha_data = read_unaligned(alphas_in);
        let color_raw = read_unaligned(colors_in);
        let index_value = read_unaligned(indices_in);

        alphas_in = alphas_in.add(1);
        colors_in = colors_in.add(1);
        indices_in = indices_in.add(1);

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

        // Write BC2 block: alpha (8 bytes) + colors (4 bytes) + indices (4 bytes)
        write_unaligned(output_ptr as *mut u64, alpha_data);
        write_unaligned(output_ptr.add(8) as *mut u32, recorrelated_colors);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value);

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

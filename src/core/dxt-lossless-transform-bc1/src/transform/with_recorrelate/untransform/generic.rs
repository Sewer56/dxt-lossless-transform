use core::hint::unreachable_unchecked;
use core::ptr::read_unaligned;
use core::ptr::write_unaligned;
use dxt_lossless_transform_common::color_565::Color565;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[inline]
pub(crate) unsafe fn untransform_with_recorrelate_generic(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr<const VARIANT: u8>(
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    let colors_end = colors_in.add(num_blocks);
    while colors_in < colors_end {
        // Read both values first (better instruction scheduling)
        let color_raw = read_unaligned(colors_in);
        let index_value = read_unaligned(indices_in);

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

        // Write both values together
        write_unaligned(output_ptr as *mut u32, recorrelated_colors);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        output_ptr = output_ptr.add(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: WithRecorrelateUntransformFn,
        #[case] decorr_variant: YCoCgVariant,
    ) {
        run_with_recorrelate_untransform_unaligned_test(
            function,
            decorr_variant,
            "unaligned untransform with recorrelation (generic)",
            2, // 8 bytes tested per main loop iteration (* 2 / 8 == 2)
        );
    }
}

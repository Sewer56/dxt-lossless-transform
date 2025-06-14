use crate::transforms::standard::untransform::unsplit_block_with_separate_pointers;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::{
    allocate::allocate_align_64,
    color_565::{Color565, YCoCgVariant},
};

pub(crate) unsafe fn untransform_with_split_colour_and_recorr_generic(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    // Note(sewer): I can't get good generic codegen for this at the moment,
    //              on x86, the codegen is poor, moving values in and out of SIMD registers
    //              for the recorrelation step.
    //
    //              I'm guessing the same will be the case on aarch64 (don't have a high end aarch64
    //              machine to test on). So I've opted for another solution for the time being, that
    //              uses 2 functions (which do get correctly optimized) and an intermediate work buffer.
    //
    //              This will make the function limited by memory bandwidth, due to extra copy.
    //              For x86 at least, I got custom intrinsic functions, to overcome this.

    // Allocating here has some overhead, so we'll delegate to the slower solution if under 512 bytes.
    // 128 blocks is 1024 bytes
    if block_count >= 128 {
        let mut work_alloc =
            allocate_align_64(block_count * 8).expect("Failed to allocate work buffer");
        let work_ptr = work_alloc.as_mut_ptr();

        // Recorrelate colours into work area, doing the unsplit in the same process.
        Color565::recorrelate_ycocg_r_ptr_split(
            color0_in as *mut Color565,
            color1_in as *mut Color565,
            work_ptr as *mut Color565,
            block_count * 2, // 2 colour endpoints per block.
            recorrelation_mode,
        );

        // Now unsplit the colours, placing them into the final buffer
        unsplit_block_with_separate_pointers(
            work_ptr as *const u32,
            indices_in,
            output_ptr,
            block_count * 8,
        );
        return;
    }

    match recorrelation_mode {
        YCoCgVariant::None => unreachable_unchecked(),
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
pub(crate) unsafe fn untransform_recorr_var1(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<1>(color0_in, color1_in, indices_in, output_ptr, block_count)
}

pub(crate) unsafe fn untransform_recorr_var2(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<2>(color0_in, color1_in, indices_in, output_ptr, block_count)
}

pub(crate) unsafe fn untransform_recorr_var3(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<3>(color0_in, color1_in, indices_in, output_ptr, block_count)
}

unsafe fn untransform_recorr<const VARIANT: u8>(
    mut color0_in: *const u16,
    mut color1_in: *const u16,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    let color0_end = color0_in.add(block_count);
    while color0_in < color0_end {
        // Read the correlated colors and apply recorrelation using the specified variant
        let color0_obj = Color565::from_raw(color0_in.read_unaligned());
        let color1_obj = Color565::from_raw(color1_in.read_unaligned());
        let (recorrelated_color0, recorrelated_color1) = match VARIANT {
            1 => (
                color0_obj.recorrelate_ycocg_r_var1(),
                color1_obj.recorrelate_ycocg_r_var1(),
            ),
            2 => (
                color0_obj.recorrelate_ycocg_r_var2(),
                color1_obj.recorrelate_ycocg_r_var2(),
            ),
            3 => (
                color0_obj.recorrelate_ycocg_r_var3(),
                color1_obj.recorrelate_ycocg_r_var3(),
            ),
            _ => unreachable_unchecked(),
        };

        // Read the indices
        let indices = indices_in.read_unaligned();

        // Write the BC1 block directly: color0 (2 bytes) + color1 (2 bytes) + indices (4 bytes)
        // Colors are stored in little-endian format as u16 values
        (output_ptr as *mut u16).write_unaligned(recorrelated_color0.raw_value());
        (output_ptr.add(2) as *mut u16).write_unaligned(recorrelated_color1.raw_value());
        (output_ptr.add(4) as *mut u32).write_unaligned(indices);

        // Advance pointers
        color0_in = color0_in.add(1);
        color1_in = color1_in.add(1);
        indices_in = indices_in.add(1);
        output_ptr = output_ptr.add(8); // 8 bytes per BC1 block
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
    fn can_untransform_unaligned(#[case] decorr_variant: YCoCgVariant) {
        // 1 block processed per iteration (* 2 == 2)
        run_with_split_colour_and_recorr_generic_untransform_unaligned_test(
            untransform_with_split_colour_and_recorr_generic,
            decorr_variant,
            2,
            "untransform_with_split_colour_and_recorr (generic, unaligned)",
        );
    }
}

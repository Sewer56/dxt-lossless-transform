use core::hint::unreachable_unchecked;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

/// Generic fallback implementation of split-colour transform with YCoCg-R decorrelation.
///
/// Splits standard interleaved BC1 blocks into separate colour0, colour1 and indices buffers,
/// applying the selected YCoCg-R decorrelation variant to each colour endpoint.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_decorr_generic(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            transform_split_decorr::<1>(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::Variant2 => {
            transform_split_decorr::<2>(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::Variant3 => {
            transform_split_decorr::<3>(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

unsafe fn transform_split_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut color0_ptr: *mut u16,
    mut color1_ptr: *mut u16,
    mut indices_ptr: *mut u32,
    block_count: usize,
) {
    let input_end = input_ptr.add(block_count * 8);
    while input_ptr < input_end {
        // Read BC1 block (colour0, colour1, indices)
        let color0_raw = read_unaligned(input_ptr as *const u16);
        let color1_raw = read_unaligned(input_ptr.add(2) as *const u16);
        let indices = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Convert raw values to Color565 and decorrelate
        let color0 = Color565::from_raw(color0_raw);
        let color1 = Color565::from_raw(color1_raw);

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

        // Write decorrelated endpoints and indices
        write_unaligned(color0_ptr, decorr0.raw_value());
        write_unaligned(color1_ptr, decorr1.raw_value());
        write_unaligned(indices_ptr, indices);

        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour_and_recorr::untransform::untransform_with_split_colour_and_recorr;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn generic_transform_roundtrip(#[case] variant: YCoCgVariant) {
        for blocks in 1..=256 {
            let input = generate_bc1_test_data(blocks);
            let mut colour0 = vec![0u16; blocks];
            let mut colour1 = vec![0u16; blocks];
            let mut indices = vec![0u32; blocks];
            let mut reconstructed = vec![0u8; input.len()];
            unsafe {
                transform_with_split_colour_and_decorr_generic(
                    input.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    blocks,
                    variant,
                );
                untransform_with_split_colour_and_recorr(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    blocks,
                    variant,
                );
            }
            assert_eq!(
                input.as_slice(),
                reconstructed.as_slice(),
                "Generic roundtrip mismatch for {variant:?}"
            );
        }
    }
}

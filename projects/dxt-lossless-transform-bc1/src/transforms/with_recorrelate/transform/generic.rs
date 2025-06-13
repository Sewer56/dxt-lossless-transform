use core::hint::unreachable_unchecked;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

/// Generic implementation of BC1 transform with YCoCg-R decorrelation.
///
/// Splits standard interleaved BC1 blocks into separate color and index buffers,
/// applying YCoCg-R decorrelation to color endpoints.
///
/// # Safety
///
/// - input_ptr must be valid for reads of num_blocks * 8 bytes
/// - colours_ptr must be valid for writes of num_blocks * 4 bytes
/// - indices_ptr must be valid for writes of num_blocks * 4 bytes
/// - decorrelation_mode must be a valid [`YCoCgVariant`]
pub(crate) unsafe fn transform_with_recorrelate_generic(
    input_ptr: *const u8,
    colours_ptr: *mut u32,
    indices_ptr: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            transform_recorr::<1>(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::Variant2 => {
            transform_recorr::<2>(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::Variant3 => {
            transform_recorr::<3>(input_ptr, colours_ptr, indices_ptr, num_blocks)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

unsafe fn transform_recorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    num_blocks: usize,
) {
    for _ in 0..num_blocks {
        // Read color endpoints and indices
        let color_raw = read_unaligned(input_ptr as *const u32);
        let index_value = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

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

        // Pack decorated colors into u32
        let decorated_colors = (decorr0.raw_value() as u32) | ((decorr1.raw_value() as u32) << 16);

        // Write to separate buffers
        write_unaligned(colours_ptr, decorated_colors);
        write_unaligned(indices_ptr, index_value);

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_recorrelate::untransform::generic::untransform_with_recorrelate_generic;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn roundtrip_transform_with_recorrelate(#[case] variant: YCoCgVariant) {
        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut transformed = vec![0u8; len];
            let mut reconstructed = vec![0u8; len];
            unsafe {
                transform_with_recorrelate_generic(
                    input.as_ptr(),
                    transformed.as_mut_ptr() as *mut u32,
                    transformed.as_mut_ptr().add(len / 2) as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_with_recorrelate_generic(
                    transformed.as_ptr() as *const u32,
                    transformed.as_ptr().add(len / 2) as *const u32,
                    reconstructed.as_mut_ptr(),
                    num_blocks,
                    variant,
                );
            }
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "roundtrip mismatch for variant {variant:?}"
            );
        }
    }
}

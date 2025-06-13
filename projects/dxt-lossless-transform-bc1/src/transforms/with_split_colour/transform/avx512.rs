use crate::transforms::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// AVX512 implementation for split-colour transform.
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    // TODO: implement AVX512 optimized transform
    generic::transform_with_split_colour(
        input_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
    );
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour::untransform::untransform_with_split_colour;

    #[rstest]
    fn avx512_transform_roundtrip() {
        for num_blocks in 1..=128 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut colour0 = vec![0u16; num_blocks];
            let mut colour1 = vec![0u16; num_blocks];
            let mut indices = vec![0u32; num_blocks];
            let mut reconstructed = vec![0u8; len];
            unsafe {
                transform_with_split_colour(
                    input.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    num_blocks,
                );
                untransform_with_split_colour(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    num_blocks,
                );
            }
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "Mismatch AVX512 roundtrip split-colour",
            );
        }
    }
}
*/

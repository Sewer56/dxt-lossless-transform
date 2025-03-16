use crate::color_565::Color565;
use std::mem::size_of;

/// Splits the colour endpoints using 64-bit operations
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len` must be a multiple of 4
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Cast input/output to u64 pointers for direct value access
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u16;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u16;

    while input < max_input_ptr {
        // Read color0 and color1 (interleaved in input)
        let color0 = *input;
        input = input.add(1);

        // Extract and write four 2-byte values from the 8-byte chunk
        *output0 = get_first2bytes(color0);
        output0 = output0.add(1);
        *output1 = get_second2bytes(color0);
        output1 = output1.add(1);

        *output0 = get_third2bytes(color0);
        output0 = output0.add(1);
        *output1 = get_fourth2bytes(color0);
        output1 = output1.add(1);
    }
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_first2bytes(value: u64) -> u16 {
    (value >> 48) as u16
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_second2bytes(value: u64) -> u16 {
    (value >> 32) as u16
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_third2bytes(value: u64) -> u16 {
    (value >> 16) as u16
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_fourth2bytes(value: u64) -> u16 {
    (value) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_first2bytes(value: u64) -> u16 {
    (value) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_second2bytes(value: u64) -> u16 {
    (value >> 16) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_third2bytes(value: u64) -> u16 {
    (value >> 32) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_fourth2bytes(value: u64) -> u16 {
    (value >> 48) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_color_endpoints::tests::{
        generate_test_data, transform_with_reference_implementation,
    };
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::min_size(4)] // 8 bytes - 4 color pairs
    #[case::one_unroll(8)] // 16 bytes - 8 color pairs
    #[case::many_unrolls(64)] // 128 bytes - tests multiple unroll iterations
    #[case::large(512)] // 1024 bytes - large dataset
    fn test_implementations(#[case] num_pairs: usize) {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test the u64 implementation
        let implementations: [(&str, TransformFn); 1] = [("u64", u64)];

        for (impl_name, implementation) in implementations {
            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_eq!(
                output_expected, output_test,
                "{} implementation produced different results than reference for {} color pairs.\n\
                First differing pair will have predictable values:\n\
                Color0: Sequential bytes 0x00,0x01 + (pair_num * 4)\n\
                Color1: Sequential bytes 0x80,0x81 + (pair_num * 4)",
                impl_name, num_pairs
            );
        }
    }
}

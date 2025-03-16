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
        *output1 = get_second2bytes(color0);
        *output0.add(1) = get_third2bytes(color0);
        *output1.add(1) = get_fourth2bytes(color0);
        output0 = output0.add(2);
        output1 = output1.add(2);
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 2
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
pub unsafe fn u64_unroll_2(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_2"
    );

    // Cast input/output to u64 pointers for direct value access
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u16;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u16;

    while input < max_input_ptr.sub(1) {
        // Process 2 chunks per iteration
        let color0 = *input;
        let color1 = *input.add(1);
        input = input.add(2);

        // Process first chunk
        *output0 = get_first2bytes(color0);
        *output0.add(2) = get_first2bytes(color1);
        *output1 = get_second2bytes(color0);
        *output1.add(2) = get_second2bytes(color1);
        *output0.add(1) = get_third2bytes(color0);
        *output0.add(3) = get_third2bytes(color1);
        *output1.add(1) = get_fourth2bytes(color0);
        *output1.add(3) = get_fourth2bytes(color1);

        output0 = output0.add(4);
        output1 = output1.add(4);
    }

    // Handle any remaining elements (shouldn't happen with proper alignment)
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        *output0 = get_first2bytes(color0);
        *output1 = get_second2bytes(color0);
        *output0.add(1) = get_third2bytes(color0);
        *output1.add(1) = get_fourth2bytes(color0);
        output0 = output0.add(2);
        output1 = output1.add(2);
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 4
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
pub unsafe fn u64_unroll_4(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 16 && colors_len_bytes % 16 == 0,
        "colors_len_bytes must be at least 16 and a multiple of 16 for unroll_4"
    );

    // Cast input/output to u64 pointers for direct value access
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u16;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u16;

    while input < max_input_ptr.sub(3) {
        // Process 4 chunks per iteration
        let color0 = *input;
        let color1 = *input.add(1);
        let color2 = *input.add(2);
        let color3 = *input.add(3);
        input = input.add(4);

        // Process first chunk
        *output0 = get_first2bytes(color0);
        *output0.add(2) = get_first2bytes(color1);
        *output0.add(4) = get_first2bytes(color2);
        *output0.add(6) = get_first2bytes(color3);
        *output1.add(2) = get_second2bytes(color1);
        *output1 = get_second2bytes(color0);
        *output1.add(4) = get_second2bytes(color2);
        *output1.add(6) = get_second2bytes(color3);
        *output0.add(1) = get_third2bytes(color0);
        *output0.add(3) = get_third2bytes(color1);
        *output0.add(5) = get_third2bytes(color2);
        *output0.add(7) = get_third2bytes(color3);
        *output1.add(1) = get_fourth2bytes(color0);
        *output1.add(3) = get_fourth2bytes(color1);
        *output1.add(5) = get_fourth2bytes(color2);
        *output1.add(7) = get_fourth2bytes(color3);

        output0 = output0.add(8);
        output1 = output1.add(8);
    }

    // Handle any remaining elements
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        *output0 = get_first2bytes(color0);
        *output1 = get_second2bytes(color0);
        *output0.add(1) = get_third2bytes(color0);
        *output1.add(1) = get_fourth2bytes(color0);
        output0 = output0.add(2);
        output1 = output1.add(2);
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 8
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
pub unsafe fn u64_unroll_8(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 32 && colors_len_bytes % 32 == 0,
        "colors_len_bytes must be at least 32 and a multiple of 32 for unroll_8"
    );

    // Cast input/output to u64 pointers for direct value access
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u16;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u16;

    while input < max_input_ptr.sub(7) {
        // Process 8 chunks per iteration
        let color0 = *input;
        let color1 = *input.add(1);
        let color2 = *input.add(2);
        let color3 = *input.add(3);
        let color4 = *input.add(4);
        let color5 = *input.add(5);
        let color6 = *input.add(6);
        let color7 = *input.add(7);
        input = input.add(8);

        // Process chunks
        *output0 = get_first2bytes(color0);
        *output0.add(2) = get_first2bytes(color1);
        *output0.add(4) = get_first2bytes(color2);
        *output0.add(6) = get_first2bytes(color3);
        *output0.add(8) = get_first2bytes(color4);
        *output0.add(10) = get_first2bytes(color5);
        *output0.add(12) = get_first2bytes(color6);
        *output0.add(14) = get_first2bytes(color7);

        *output1 = get_second2bytes(color0);
        *output1.add(2) = get_second2bytes(color1);
        *output1.add(4) = get_second2bytes(color2);
        *output1.add(6) = get_second2bytes(color3);
        *output1.add(8) = get_second2bytes(color4);
        *output1.add(10) = get_second2bytes(color5);
        *output1.add(12) = get_second2bytes(color6);
        *output1.add(14) = get_second2bytes(color7);

        *output0.add(1) = get_third2bytes(color0);
        *output0.add(3) = get_third2bytes(color1);
        *output0.add(5) = get_third2bytes(color2);
        *output0.add(7) = get_third2bytes(color3);
        *output0.add(9) = get_third2bytes(color4);
        *output0.add(11) = get_third2bytes(color5);
        *output0.add(15) = get_third2bytes(color7);
        *output0.add(13) = get_third2bytes(color6);

        *output1.add(1) = get_fourth2bytes(color0);
        *output1.add(3) = get_fourth2bytes(color1);
        *output1.add(5) = get_fourth2bytes(color2);
        *output1.add(7) = get_fourth2bytes(color3);
        *output1.add(9) = get_fourth2bytes(color4);
        *output1.add(11) = get_fourth2bytes(color5);
        *output1.add(13) = get_fourth2bytes(color6);
        *output1.add(15) = get_fourth2bytes(color7);

        output0 = output0.add(16);
        output1 = output1.add(16);
    }

    // Handle any remaining elements
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        *output0 = get_first2bytes(color0);
        *output1 = get_second2bytes(color0);
        *output0.add(1) = get_third2bytes(color0);
        *output1.add(1) = get_fourth2bytes(color0);
        output0 = output0.add(2);
        output1 = output1.add(2);
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
    #[case::many_unrolls(64)] // 128 bytes - tests multiple unroll iterations
    #[case::large(512)] // 1024 bytes - large dataset
    fn test_implementations(#[case] num_pairs: usize) {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test the u64 implementation
        let implementations: [(&str, TransformFn); 4] = [
            ("u64", u64),
            ("u64_unroll_2", u64_unroll_2),
            ("u64_unroll_4", u64_unroll_4),
            ("u64_unroll_8", u64_unroll_8),
        ];

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

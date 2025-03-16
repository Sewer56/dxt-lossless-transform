use crate::color_565::Color565;
use std::mem::size_of;

/// Splits the colour endpoints
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
/// - Pointers must be properly aligned for u32 access
pub unsafe fn portable_32(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    // Debug assert: colors_len_bytes must be at least 4 and a multiple of 4
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Cast input/output to u32 pointers for direct value access
    let max_input_ptr = colors.add(colors_len_bytes) as *const u32;
    let mut input = colors as *const u32;
    let mut output0 = colors_out as *mut u16;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u16;

    while input < max_input_ptr {
        // Read color0 and color1 (interleaved in input)
        let color0 = *input;
        input = input.add(1);
        *output0 = get_first2bytes(color0);
        output0 = output0.add(1);
        *output1 = get_second2bytes(color0);
        output1 = output1.add(1);
    }
}

#[cfg(target_endian = "big")]
#[cfg(target_endian = "big")]
#[inline(always)]
fn get_second2bytes(value: u32) -> u16 {
    (value) as u16
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_first2bytes(value: u32) -> u16 {
    (value >> 16) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_second2bytes(value: u32) -> u16 {
    (value >> 16) as u16
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_first2bytes(value: u32) -> u16 {
    (value) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Transforms the input data using a known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe {
            portable_32(input.as_ptr(), output.as_mut_ptr(), input.len());
        }
    }

    // Helper to generate test data of specified size (in color pairs)
    pub(crate) fn generate_test_data(num_pairs: usize) -> Vec<u8> {
        let mut data = Vec::with_capacity(num_pairs * 4); // Each pair is 4 bytes (2 u16 values)

        let mut color0_byte = 0_u8;
        let mut color1_byte = 128_u8;

        for _ in 0..num_pairs {
            // Color0: 2 bytes
            data.push(color0_byte);
            data.push(color0_byte.wrapping_add(1));

            // Color1: 2 bytes
            data.push(color1_byte);
            data.push(color1_byte.wrapping_add(1));

            color0_byte = color0_byte.wrapping_add(4);
            color1_byte = color1_byte.wrapping_add(4);
        }

        data
    }

    #[test]
    fn test_reference_implementation() {
        let input: Vec<u8> = vec![
            0x00, 0x01, // pair 1 color 0
            0x10, 0x11, // pair 1 color 1
            0x04, 0x05, // pair 2 color 0
            0x14, 0x15, // pair 2 color 1
            0x08, 0x09, // pair 3 color 0
            0x18, 0x19, // pair 3 color 1
        ];
        let mut output = vec![0u8; 12];

        transform_with_reference_implementation(&input, &mut output);

        assert_eq!(
            output,
            vec![
                0x00, 0x01, // colors: pair 1 color 0
                0x04, 0x05, // colors: pair 2 color 0
                0x08, 0x09, // colors: pair 3 color 0
                0x10, 0x11, // colors: pair 1 color 1
                0x14, 0x15, // colors: pair 2 color 1
                0x18, 0x19, // colors: pair 3 color 1
            ]
        );
    }

    #[test]
    fn validate_test_data_generator() {
        let expected: Vec<u8> = vec![
            0x00, 0x01, // pair 1 color 0
            0x80, 0x81, // pair 1 color 1
            0x04, 0x05, // pair 2 color 0
            0x84, 0x85, // pair 2 color 1
            0x08, 0x09, // pair 3 color 0
            0x88, 0x89, // pair 3 color 1
        ];

        let output = generate_test_data(3);

        assert_eq!(output, expected);
    }
}

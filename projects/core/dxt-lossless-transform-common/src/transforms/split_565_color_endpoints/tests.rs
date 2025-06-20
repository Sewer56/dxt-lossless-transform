//! Common test utilities for split_565_color_endpoints implementations
//!
//! This module provides shared test infrastructure to reduce code duplication
//! across different SIMD implementation test modules.

use super::portable32;

/// Function pointer type for split color endpoints implementations
pub type TransformFn = unsafe fn(*const u8, *mut u8, usize);

/// Transforms the input data using a known reference implementation.
pub fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
    unsafe {
        portable32::u32(input.as_ptr(), output.as_mut_ptr(), input.len());
    }
}

/// Helper to generate test data of specified size (in color pairs)
pub fn generate_test_data(num_pairs: usize) -> Vec<u8> {
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

        color0_byte = color0_byte.wrapping_add(2);
        color1_byte = color1_byte.wrapping_add(2);
    }

    data
}

/// Helper to assert implementation results match reference implementation
pub fn assert_implementation_matches_reference(
    output_expected: &[u8],
    output_test: &[u8],
    impl_name: &str,
    num_pairs: usize,
) {
    assert_eq!(
        output_expected, output_test,
        "{impl_name} implementation produced different results than reference for {num_pairs} color pairs.\n\
        First differing pair will have predictable values:\n\
        Color0: Sequential bytes 0x00,0x01 + (pair_num * 4)\n\
        Color1: Sequential bytes 0x80,0x81 + (pair_num * 4)"
    );
}

/// Test an implementation with aligned buffers for a range of input sizes
pub fn test_implementation_aligned(implementation: TransformFn, impl_name: &str) {
    for num_pairs in 1..=512 {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Clear the output buffer
        output_test.fill(0);

        // Run the implementation
        unsafe {
            implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
        }

        // Compare results
        assert_implementation_matches_reference(
            &output_expected,
            &output_test,
            &format!("{impl_name} (aligned)"),
            num_pairs,
        );
    }
}

/// Test an implementation with unaligned buffers for a range of input sizes
pub fn test_implementation_unaligned(implementation: TransformFn, impl_name: &str) {
    for num_pairs in 1..=512 {
        let input = generate_test_data(num_pairs);

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut input_unaligned = vec![0u8; input.len() + 1];
        input_unaligned[1..].copy_from_slice(input.as_slice());

        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len() + 1];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Clear the output buffer
        output_test.fill(0);

        // Run the implementation
        unsafe {
            // Use pointers offset by 1 byte to create unaligned access
            implementation(
                input_unaligned.as_ptr().add(1),
                output_test.as_mut_ptr().add(1),
                input.len(),
            );
        }

        // Compare results
        assert_implementation_matches_reference(
            &output_expected,
            &output_test[1..],
            &format!("{impl_name} (unaligned)"),
            num_pairs,
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

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
            0x02, 0x03, // pair 2 color 0
            0x82, 0x83, // pair 2 color 1
            0x04, 0x05, // pair 3 color 0
            0x84, 0x85, // pair 3 color 1
        ];

        let output = generate_test_data(3);

        assert_eq!(output, expected);
    }
}

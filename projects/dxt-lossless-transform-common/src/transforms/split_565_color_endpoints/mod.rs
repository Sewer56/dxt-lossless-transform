//! # Splitting the Colour Endpoints
//!
//! Each BC1 texture has 2 colour endpoints, `color0` and `color1`.
//! It is sometimes beneficial to separate these, i.e. store them separately.
//!
//! Take our optimized layout from earlier:
//!
//! ```text
//! +-------+-------+-------+
//! |C0  C1 |C0  C1 |C0  C1 |
//! +-------+-------+-------+
//! ```
//!
//! We can split the colour endpoints
//!
//! ```text
//! +-------+-------+ +-------+-------+
//! |C0  C0 |C0  C0 | |C1  C1 |C1  C1 |
//! +-------+-------+ +-------+-------+
//! ```

pub(crate) mod portable32;
mod portable64;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod ssse3;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod avx512;

#[cfg(feature = "bench")]
pub mod bench;

use crate::color_565::Color565;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_color_endpoints_x86(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if crate::cpu_detect::has_avx512vbmi() {
            avx512::avx512_impl(colors, colors_out, colors_len_bytes);
            return;
        }

        if crate::cpu_detect::has_avx2() {
            avx2::avx2_shuf_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }

        if crate::cpu_detect::has_sse2() {
            sse2::sse2_shuf_unroll2_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_impl(colors, colors_out, colors_len_bytes);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::avx2_shuf_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::sse2_shuf_unroll2_impl_asm(colors, colors_out, colors_len_bytes);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32(colors, colors_out, colors_len_bytes)
}

/// Splits the colour endpoints using the best known implementation for the current CPU.
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
/// - `colors_len_bytes` must be a multiple of 4
///
/// # Remarks
///
/// For performance it's recommended that colors and colors_out are 32-byte aligned.
/// As this method may use a SIMD implementation.
#[inline]
pub unsafe fn split_color_endpoints(
    colors: *const Color565,
    colors_out: *mut Color565,
    colors_len_bytes: usize,
) {
    // Debug assert: colors_len_bytes must be at least 4 and a multiple of 4
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_color_endpoints_x86(colors as *const u8, colors_out as *mut u8, colors_len_bytes)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32(colors as *const u8, colors_out as *mut u8, colors_len_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Transforms the input data using a known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe {
            portable32::u32(input.as_ptr(), output.as_mut_ptr(), input.len());
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

            color0_byte = color0_byte.wrapping_add(2);
            color1_byte = color1_byte.wrapping_add(2);
        }

        data
    }

    /// Helper to assert implementation results match reference implementation
    pub(crate) fn assert_implementation_matches_reference(
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

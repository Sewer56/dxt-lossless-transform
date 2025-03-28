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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8"
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
/// - `colors_len` must be a multiple of 8
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_unroll_4(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_4"
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
        *output1 = get_second2bytes(color0);
        *output1.add(2) = get_second2bytes(color1);
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_unroll_8(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_8"
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
        *output0.add(13) = get_third2bytes(color6);
        *output0.add(15) = get_third2bytes(color7);

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

/// Splits the colour endpoints using 64-bit operations, but writing with mixed (combined) 32-bit values
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_mix(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8"
    );

    // Cast input/output to appropriate pointer types
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u32;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u32;

    while input < max_input_ptr {
        // Read color0 and color1 (interleaved in input)
        let color0 = *input;
        input = input.add(1);

        // Combine first and third 2-bytes into a single u32
        let first_pair = combine_u16_pair_u32(get_first2bytes(color0), get_third2bytes(color0));

        // Combine second and fourth 2-bytes into a single u32
        let second_pair = combine_u16_pair_u32(get_second2bytes(color0), get_fourth2bytes(color0));

        // Write the combined values
        *output0 = first_pair;
        *output1 = second_pair;

        output0 = output0.add(1);
        output1 = output1.add(1);
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 2,
/// writing with 64-bit values
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_mix_unroll_2(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_2"
    );

    // Cast input/output to appropriate pointer types
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u64;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u64;

    while input < max_input_ptr.sub(1) {
        // Process 2 chunks per iteration
        let color0 = *input;
        let color1 = *input.add(1);
        input = input.add(2);

        // Combine the values using helper functions
        let first_combined = combine_u16_quad_u64(
            get_first2bytes(color0),
            get_third2bytes(color0),
            get_first2bytes(color1),
            get_third2bytes(color1),
        );

        let second_combined = combine_u16_quad_u64(
            get_second2bytes(color0),
            get_fourth2bytes(color0),
            get_second2bytes(color1),
            get_fourth2bytes(color1),
        );

        // Write the combined values as u64s
        *output0 = first_combined;
        *output1 = second_combined;

        output0 = output0.add(1);
        output1 = output1.add(1);
    }

    // Handle any remaining elements (shouldn't happen with proper alignment)
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        // For a single remaining item, we need to write as 32-bit since we
        // don't have enough data for a full 64-bit write
        let first_pair = combine_u16_pair_u32(get_first2bytes(color0), get_third2bytes(color0));
        let second_pair = combine_u16_pair_u32(get_second2bytes(color0), get_fourth2bytes(color0));

        // Write the combined values to 32-bit locations
        *(output0 as *mut u32) = first_pair;
        *(output1 as *mut u32) = second_pair;

        // Move pointers forward by half a u64
        output0 = (output0 as *mut u32).add(1) as *mut u64;
        output1 = (output1 as *mut u32).add(1) as *mut u64;
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 4,
/// writing with 64-bit values.
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_mix_unroll_4(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_4"
    );

    // Cast input/output to appropriate pointer types
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u64;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u64;

    while input < max_input_ptr.sub(3) {
        // Process 4 chunks per iteration
        let color0 = *input;
        let color1 = *input.add(1);
        let color2 = *input.add(2);
        let color3 = *input.add(3);
        input = input.add(4);

        // Combine the first two colors
        let first_combined0 = combine_u16_quad_u64(
            get_first2bytes(color0),
            get_third2bytes(color0),
            get_first2bytes(color1),
            get_third2bytes(color1),
        );

        let first_combined1 = combine_u16_quad_u64(
            get_first2bytes(color2),
            get_third2bytes(color2),
            get_first2bytes(color3),
            get_third2bytes(color3),
        );

        *output0 = first_combined0;
        *output0.add(1) = first_combined1;

        let second_combined0 = combine_u16_quad_u64(
            get_second2bytes(color0),
            get_fourth2bytes(color0),
            get_second2bytes(color1),
            get_fourth2bytes(color1),
        );

        let second_combined1 = combine_u16_quad_u64(
            get_second2bytes(color2),
            get_fourth2bytes(color2),
            get_second2bytes(color3),
            get_fourth2bytes(color3),
        );

        *output1 = second_combined0;
        *output1.add(1) = second_combined1;

        output0 = output0.add(2);
        output1 = output1.add(2);
    }

    // Handle any remaining elements
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        // For a single remaining item, write as 32-bit
        let first_pair = combine_u16_pair_u32(get_first2bytes(color0), get_third2bytes(color0));
        let second_pair = combine_u16_pair_u32(get_second2bytes(color0), get_fourth2bytes(color0));

        // Write the combined values to 32-bit locations
        *(output0 as *mut u32) = first_pair;
        *(output1 as *mut u32) = second_pair;

        // Move pointers forward by half a u64
        output0 = (output0 as *mut u32).add(1) as *mut u64;
        output1 = (output1 as *mut u32).add(1) as *mut u64;
    }
}

/// Splits the colour endpoints using 64-bit operations with loop unrolling factor of 8,
/// writing with 64-bit values.
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
/// - `colors_len` must be a multiple of 8
/// - Pointers must be properly aligned for u64 access
#[inline(always)]
pub unsafe fn u64_mix_unroll_8(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 8 && colors_len_bytes % 8 == 0,
        "colors_len_bytes must be at least 8 and a multiple of 8 for unroll_8"
    );

    // Cast input/output to appropriate pointer types
    let max_input_ptr = colors.add(colors_len_bytes) as *const u64;
    let mut input = colors as *const u64;
    let mut output0 = colors_out as *mut u64;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u64;

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

        // First pair of colors
        let first_combined0 = combine_u16_quad_u64(
            get_first2bytes(color0),
            get_third2bytes(color0),
            get_first2bytes(color1),
            get_third2bytes(color1),
        );

        let first_combined1 = combine_u16_quad_u64(
            get_first2bytes(color2),
            get_third2bytes(color2),
            get_first2bytes(color3),
            get_third2bytes(color3),
        );

        let first_combined2 = combine_u16_quad_u64(
            get_first2bytes(color4),
            get_third2bytes(color4),
            get_first2bytes(color5),
            get_third2bytes(color5),
        );

        let first_combined3 = combine_u16_quad_u64(
            get_first2bytes(color6),
            get_third2bytes(color6),
            get_first2bytes(color7),
            get_third2bytes(color7),
        );

        *output0 = first_combined0;
        *output0.add(1) = first_combined1;
        *output0.add(2) = first_combined2;
        *output0.add(3) = first_combined3;

        let second_combined0 = combine_u16_quad_u64(
            get_second2bytes(color0),
            get_fourth2bytes(color0),
            get_second2bytes(color1),
            get_fourth2bytes(color1),
        );

        let second_combined1 = combine_u16_quad_u64(
            get_second2bytes(color2),
            get_fourth2bytes(color2),
            get_second2bytes(color3),
            get_fourth2bytes(color3),
        );

        let second_combined2 = combine_u16_quad_u64(
            get_second2bytes(color4),
            get_fourth2bytes(color4),
            get_second2bytes(color5),
            get_fourth2bytes(color5),
        );

        let second_combined3 = combine_u16_quad_u64(
            get_second2bytes(color6),
            get_fourth2bytes(color6),
            get_second2bytes(color7),
            get_fourth2bytes(color7),
        );

        *output1 = second_combined0;
        *output1.add(1) = second_combined1;
        *output1.add(2) = second_combined2;
        *output1.add(3) = second_combined3;

        output0 = output0.add(4);
        output1 = output1.add(4);
    }

    // Handle any remaining elements
    while input < max_input_ptr {
        let color0 = *input;
        input = input.add(1);

        // For a single remaining item, write as 32-bit
        let first_pair = combine_u16_pair_u32(get_first2bytes(color0), get_third2bytes(color0));
        let second_pair = combine_u16_pair_u32(get_second2bytes(color0), get_fourth2bytes(color0));

        // Write the combined values to 32-bit locations
        *(output0 as *mut u32) = first_pair;
        *(output1 as *mut u32) = second_pair;

        // Move pointers forward by half a u64
        output0 = (output0 as *mut u32).add(1) as *mut u64;
        output1 = (output1 as *mut u32).add(1) as *mut u64;
    }
}

/// Helper function to combine two u16 values into a u32
#[cfg(target_endian = "little")]
#[inline(always)]
fn combine_u16_pair_u32(low: u16, high: u16) -> u32 {
    (low as u32) | ((high as u32) << 16)
}

/// Helper function to combine four u16 values into a u64
#[cfg(target_endian = "little")]
#[inline(always)]
fn combine_u16_quad_u64(first: u16, second: u16, third: u16, fourth: u16) -> u64 {
    (first as u64) | ((second as u64) << 16) | ((third as u64) << 32) | ((fourth as u64) << 48)
}

/// Helper function to combine two u16 values into a u32 for big endian
#[cfg(target_endian = "big")]
#[inline(always)]
fn combine_u16_pair_u32(low: u16, high: u16) -> u32 {
    ((low as u32) << 16) | (high as u32)
}

/// Helper function to combine four u16 values into a u64 for big endian
#[cfg(target_endian = "big")]
#[inline(always)]
fn combine_u16_quad_u64(first: u16, second: u16, third: u16, fourth: u16) -> u64 {
    ((first as u64) << 48) | ((second as u64) << 32) | ((third as u64) << 16) | (fourth as u64)
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
    #[case::single(4)] // 8 bytes - single iteration
    #[case::many_unrolls(64)] // 128 bytes - tests multiple unroll iterations
    #[case::large(512)] // 1024 bytes - large dataset
    fn test_implementations(#[case] num_pairs: usize) {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test the u64 implementation
        let implementations: [(&str, TransformFn); 8] = [
            ("u64", u64),
            ("u64_unroll_2", u64_unroll_2),
            ("u64_unroll_4", u64_unroll_4),
            ("u64_unroll_8", u64_unroll_8),
            ("u64_mix", u64_mix),
            ("u64_mix_unroll_2", u64_mix_unroll_2),
            ("u64_mix_unroll_4", u64_mix_unroll_4),
            ("u64_mix_unroll_8", u64_mix_unroll_8),
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

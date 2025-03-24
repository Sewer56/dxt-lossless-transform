use crate::color_565::Color565;
use std::mem::size_of;

/// Splits the colour endpoints using 32-bit operations
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
/// - Pointers must be properly aligned for u32 access
#[inline(always)]
pub unsafe fn u32(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
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

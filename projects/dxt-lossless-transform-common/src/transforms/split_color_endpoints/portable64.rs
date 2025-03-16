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
    let mut output0 = colors_out as *mut u32;
    let mut output1 = colors_out.add(colors_len_bytes / size_of::<Color565>()) as *mut u32;

    while input < max_input_ptr {
        // Read color0 and color1 (interleaved in input)
        let color0 = *input;
        input = input.add(1);

        // Handle the first 2 pairs (4 u16 values)
        *output0 = get_lower4bytes(color0);
        output0 = output0.add(1);
        *output1 = get_upper4bytes(color0);
        output1 = output1.add(1);
    }
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_upper4bytes(value: u64) -> u32 {
    (value) as u32
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_lower4bytes(value: u64) -> u32 {
    (value >> 32) as u32
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_upper4bytes(value: u64) -> u32 {
    (value >> 32) as u32
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_lower4bytes(value: u64) -> u32 {
    (value) as u32
}

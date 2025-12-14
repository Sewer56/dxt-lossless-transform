//! Utility functions for BC3 transform operations
//!
//! This module provides endianness-safe helper functions for extracting color and alpha
//! data from u32 chunks, optimized for performance across both little and big endian systems.

/// Extract the first u16 from a u32 value (endianness-safe)
#[cfg(target_endian = "little")]
#[inline(always)]
pub(crate) fn get_first_u16_from_u32(value: u32) -> u16 {
    value as u16
}

/// Extract the second u16 from a u32 value (endianness-safe)
#[cfg(target_endian = "little")]
#[inline(always)]
pub(crate) fn get_second_u16_from_u32(value: u32) -> u16 {
    (value >> 16) as u16
}

/// Extract the first u16 from a u32 value for big endian
#[cfg(target_endian = "big")]
#[inline(always)]
pub(crate) fn get_first_u16_from_u32(value: u32) -> u16 {
    (value >> 16) as u16
}

/// Extract the second u16 from a u32 value for big endian
#[cfg(target_endian = "big")]
#[inline(always)]
pub(crate) fn get_second_u16_from_u32(value: u32) -> u16 {
    value as u16
}

/// Combine two u16 values into a u32 (endianness-safe)
#[cfg(target_endian = "little")]
#[inline(always)]
pub(crate) fn combine_u16_pair_to_u32(first: u16, second: u16) -> u32 {
    (first as u32) | ((second as u32) << 16)
}

/// Combine two u16 values into a u32 for big endian
#[cfg(target_endian = "big")]
#[inline(always)]
pub(crate) fn combine_u16_pair_to_u32(first: u16, second: u16) -> u32 {
    ((first as u32) << 16) | (second as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u16_u32_roundtrip() {
        let test_values = [
            (0x1234, 0x5678),
            (0x0000, 0xFFFF),
            (0xFFFF, 0x0000),
            (0xABCD, 0xEF01),
        ];

        for (first, second) in test_values {
            let combined = combine_u16_pair_to_u32(first, second);
            let extracted_first = get_first_u16_from_u32(combined);
            let extracted_second = get_second_u16_from_u32(combined);

            assert_eq!(first, extracted_first, "First u16 roundtrip failed");
            assert_eq!(second, extracted_second, "Second u16 roundtrip failed");
        }
    }
}

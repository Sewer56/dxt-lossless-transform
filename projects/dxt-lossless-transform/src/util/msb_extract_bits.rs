/// Extract bits from a byte using MSB (left-to-right) ordering
///
/// # Arguments
/// * `byte` - Source byte to extract from
/// * `start` - Starting bit position (0-7, where 0 is leftmost/MSB)
/// * `end` - Ending bit position (0-7, inclusive)
///
/// # Returns
/// * Extracted bits, right-aligned
///
/// # Example
/// ```
/// use dxt_lossless_transform::util::msb_extract_bits::extract_msb_bits;
/// let byte = 0b10110000;
/// assert_eq!(extract_msb_bits(byte, 0, 0), 1); // Extracts leftmost bit
/// assert_eq!(extract_msb_bits(byte, 1, 3), 0b011); // Extracts bits 1-3
/// ```
pub fn extract_msb_bits(byte: u8, start: u8, end: u8) -> u8 {
    debug_assert!(start <= end);
    debug_assert!(end <= 7);

    // Calculate number of bits to keep
    let num_bits = end - start + 1;

    // Create mask of the correct width
    let mask: u32 = (1u32 << num_bits) - 1;

    // Shift right to align target bits
    let shift = 7 - end;

    (byte >> shift) & (mask as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_extract_single_bit() {
        let byte = 0b10110000;
        assert_eq!(extract_msb_bits(byte, 0, 0), 1); // Leftmost bit
        assert_eq!(extract_msb_bits(byte, 1, 1), 0); // Second bit
        assert_eq!(extract_msb_bits(byte, 2, 2), 1); // Third bit
        assert_eq!(extract_msb_bits(byte, 7, 7), 0); // Rightmost bit
    }

    #[test]
    fn can_extract_multi_bit() {
        let byte = 0b10110001;
        assert_eq!(extract_msb_bits(byte, 0, 1), 0b10); // First two bits
        assert_eq!(extract_msb_bits(byte, 1, 3), 0b011); // Three bits from position 1
        assert_eq!(extract_msb_bits(byte, 0, 3), 0b1011); // First four bits
        assert_eq!(extract_msb_bits(byte, 4, 7), 0b0001); // Last four bits
    }

    #[test]
    fn can_extract_full_byte() {
        let byte = 0b10110000;
        assert_eq!(extract_msb_bits(byte, 0, 7), 0b10110000); // Entire byte
    }

    #[test]
    fn bc7_mode0_extraction() {
        let byte = 0b10111000; // Mode 0 with partition value 7
        assert_eq!(extract_msb_bits(byte, 0, 0), 1); // Mode bit
        assert_eq!(extract_msb_bits(byte, 1, 4), 0b0111); // Partition bits
    }
}

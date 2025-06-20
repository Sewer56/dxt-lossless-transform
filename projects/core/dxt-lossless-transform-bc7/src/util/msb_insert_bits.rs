/// Insert bits into a byte using MSB (left-to-right) ordering
///
/// # Arguments
/// * `byte` - Target byte to insert into
/// * `value` - Value to insert (must fit within specified bit range)
/// * `start` - Starting bit position (0-7, where 0 is leftmost/MSB)
/// * `end` - Ending bit position (0-7, inclusive)
///
/// # Returns
/// * New byte with inserted bits
///
/// # Example
///
/// ```no_run
/// use dxt_lossless_transform::util::msb_insert_bits::insert_msb_bits;
/// let mut byte = 0b00000000;
/// assert_eq!(insert_msb_bits(byte, 0b101, 0, 2), 0b10100000); // Insert 3 bits at start
/// assert_eq!(insert_msb_bits(byte, 0b11, 6, 7), 0b00000011); // Insert 2 bits at end
/// ```
pub(crate) fn insert_msb_bits(byte: u8, value: u8, start: u8, end: u8) -> u8 {
    debug_assert!(start <= end);
    debug_assert!(end <= 7);

    // Calculate number of bits we're inserting
    let num_bits = end - start + 1;

    // Create mask for the insertion zone
    let mask: u32 = ((1u32 << num_bits) - 1) << (7 - end);

    // Clear the target bits in the original byte
    let cleared = byte & !(mask as u8);

    // Shift the value into position
    let shift = 7 - end;
    let positioned_value = (value << shift) & (mask as u8);

    // Combine the cleared byte with the positioned value
    cleared | positioned_value
}

#[cfg(test)]
mod tests {
    use crate::util::msb_extract_bits::extract_msb_bits;

    use super::*;

    #[test]
    fn can_insert_single_bit() {
        let byte = 0b00000000;
        assert_eq!(insert_msb_bits(byte, 1, 0, 0), 0b10000000); // Leftmost bit
        assert_eq!(insert_msb_bits(byte, 1, 7, 7), 0b00000001); // Rightmost bit

        // Test inserting into non-zero byte
        let byte = 0b11111111;
        assert_eq!(insert_msb_bits(byte, 0, 3, 3), 0b11101111); // Clear middle bit
    }

    #[test]
    fn can_insert_multi_bit() {
        let byte = 0b00000000;
        assert_eq!(insert_msb_bits(byte, 0b11, 0, 1), 0b11000000); // First two bits
        assert_eq!(insert_msb_bits(byte, 0b101, 1, 3), 0b01010000); // Three bits from position 1
        assert_eq!(insert_msb_bits(byte, 0b1111, 4, 7), 0b00001111); // Last four bits

        // Test inserting into existing data
        let byte = 0b11111111;
        assert_eq!(insert_msb_bits(byte, 0b0000, 2, 5), 0b11000011); // Clear middle bits
    }

    #[test]
    fn can_insert_full_byte() {
        let byte = 0b11111111;
        assert_eq!(insert_msb_bits(byte, 0b10110000, 0, 7), 0b10110000);
    }

    #[test]
    fn can_insert_without_affecting_other_bits() {
        let byte = 0b10101010;

        // Insert in middle without affecting surrounding bits
        let result = insert_msb_bits(byte, 0b11, 3, 4);
        assert_eq!(result, 0b10111010);

        // Verify surrounding bits unchanged
        assert_eq!(extract_msb_bits(result, 0, 2), 0b101);
        assert_eq!(extract_msb_bits(result, 5, 7), 0b010);
    }
}

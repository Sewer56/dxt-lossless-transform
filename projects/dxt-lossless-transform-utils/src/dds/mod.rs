/// Determines if the given file represents a DDS archive.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0.
#[inline(always)]
pub fn is_dds(ptr: *const u8, len: usize) -> bool {
    len >= 80 && unsafe { (ptr as *const u32).read_unaligned() } == 0x20534444_u32.to_le()
}

#[cfg(test)]
mod tests {
    use core::iter::repeat;

    use super::*;

    #[test]
    fn test_is_dds() {
        // Valid DDS header with sufficient length
        let valid_data = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat(0).take(76))
            .collect::<Vec<u8>>();
        assert!(is_dds(valid_data.as_ptr(), valid_data.len()));

        // Valid header but insufficient length
        let short_valid_header = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat(0).take(75))
            .collect::<Vec<u8>>();
        assert!(!is_dds(
            short_valid_header.as_ptr(),
            short_valid_header.len()
        ));

        // Invalid header with sufficient length
        let invalid_data = repeat(0).take(80).collect::<Vec<u8>>();
        assert!(!is_dds(invalid_data.as_ptr(), invalid_data.len()));

        // Too short
        let short_data = [0x44, 0x44, 0x53, 0x20];
        assert!(!is_dds(short_data.as_ptr(), short_data.len()));

        // Empty data
        let empty_data: [u8; 0] = [];
        assert!(!is_dds(empty_data.as_ptr(), empty_data.len()));
    }
}

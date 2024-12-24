/// Determines if the given file represents a DDS archive.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0.
#[inline(always)]
pub fn is_dds(ptr: *const u8, len: usize) -> bool {
    len > 4 && unsafe { (ptr as *const u32).read_unaligned() } == 0x20534444_u32.to_le()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_dds() {
        // Valid DDS header
        let valid_data = [0x44, 0x44, 0x53, 0x20, 0x00, 0x00, 0x00, 0x00];
        assert!(is_dds(valid_data.as_ptr(), valid_data.len()));

        // Invalid header
        let invalid_data = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(!is_dds(invalid_data.as_ptr(), invalid_data.len()));

        // Too short
        let short_data = [0x44, 0x44, 0x53];
        assert!(!is_dds(short_data.as_ptr(), short_data.len()));

        // Empty data
        let empty_data: [u8; 0] = [];
        assert!(!is_dds(empty_data.as_ptr(), empty_data.len()));
    }
}

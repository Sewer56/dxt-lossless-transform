use super::constants::*;

/// Determines if the given file represents a DDS texture.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0.
#[inline(always)]
pub fn is_dds(ptr: *const u8, len: usize) -> bool {
    len >= DDS_HEADER_SIZE && unsafe { (ptr as *const u32).read_unaligned() } == DDS_MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use core::iter::repeat_n;

    #[test]
    fn is_dds_with_valid_header_and_sufficient_length() {
        let valid_data = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat_n(0, 124))
            .collect::<Vec<u8>>();
        assert!(is_dds(valid_data.as_ptr(), valid_data.len()));
    }

    #[test]
    fn is_not_dds_with_valid_header_but_insufficient_length() {
        let short_valid_header = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat_n(0, 123))
            .collect::<Vec<u8>>();
        assert!(!is_dds(
            short_valid_header.as_ptr(),
            short_valid_header.len()
        ));
    }

    #[test]
    fn is_not_dds_with_invalid_header() {
        let invalid_data = repeat_n(0, 128).collect::<Vec<u8>>();
        assert!(!is_dds(invalid_data.as_ptr(), invalid_data.len()));
    }

    #[test]
    fn is_not_dds_with_short_data() {
        let short_data = [0x44, 0x44, 0x53, 0x20];
        assert!(!is_dds(short_data.as_ptr(), short_data.len()));
    }

    #[test]
    fn is_not_dds_with_empty_data() {
        let empty_data: [u8; 0] = [];
        assert!(!is_dds(empty_data.as_ptr(), empty_data.len()));
    }
}

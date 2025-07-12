use super::constants::*;

/// Determines if the given data likely represents a DDS texture.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0 and minimum size.
/// For more accurate checking including header validation, use [`parse_dds`].
///
/// [`parse_dds`]: crate::dds::parse_dds::parse_dds
#[inline(always)]
pub fn likely_dds(data: &[u8]) -> bool {
    data.len() >= DDS_HEADER_SIZE
        && u32::from_le_bytes([data[0], data[1], data[2], data[3]]) == DDS_MAGIC
    // from_le_bytes ensures correct reading on all platforms since DDS is little-endian. No perf impact, a single cmp at runtime.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use core::iter::repeat_n;

    #[test]
    fn likely_dds_matches_valid_header_and_sufficient_length() {
        let valid_data = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat_n(0, 124))
            .collect::<Vec<u8>>();
        assert!(likely_dds(&valid_data));
    }

    #[test]
    fn likely_dds_rejects_valid_header_but_insufficient_length() {
        let short_valid_header = [0x44, 0x44, 0x53, 0x20]
            .into_iter()
            .chain(repeat_n(0, 123))
            .collect::<Vec<u8>>();
        assert!(!likely_dds(&short_valid_header));
    }

    #[test]
    fn likely_dds_rejects_invalid_header() {
        let invalid_data = repeat_n(0, 128).collect::<Vec<u8>>();
        assert!(!likely_dds(&invalid_data));
    }

    #[test]
    fn likely_dds_rejects_short_data() {
        let short_data = [0x44, 0x44, 0x53, 0x20];
        assert!(!likely_dds(&short_data));
    }

    #[test]
    fn likely_dds_rejects_empty_data() {
        let empty_data: [u8; 0] = [];
        assert!(!likely_dds(&empty_data));
    }
}

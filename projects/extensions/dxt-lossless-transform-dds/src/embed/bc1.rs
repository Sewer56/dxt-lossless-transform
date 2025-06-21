//! BC1 format embedding support.

use crate::dds::is_dds;
use bitfield::bitfield;
use dxt_lossless_transform_bc1::Bc1DetransformDetails;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

use super::EmbedError;

bitfield! {
    /// Packed BC1 transform data for embedding in DDS header.
    ///
    /// Bit layout: [reserved(21 bits)][decorrelation(2 bits)][split(1 bit)][header_version(8 bits)]
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Bc1HeaderData(u32);
    impl Debug;
    u32;

    /// Header version - combines format type and format version (8 bits)
    /// For BC1 v1: version = 0x01
    pub header_version, set_header_version: 7, 0;
    /// Whether color endpoints were split (1 bit)
    pub split_colour_endpoints, set_split_colour_endpoints: 8;
    /// YCoCg decorrelation variant (0=Variant1, 1=Variant2, 2=Variant3, 3=None) (2 bits)
    pub decorrelation_variant, set_decorrelation_variant: 10, 9;
    /// Reserved bits for future use (21 bits)
    pub reserved, set_reserved: 31, 11;
}

impl Bc1HeaderData {
    /// BC1 format version 1 header version identifier
    const BC1_V1_HEADER_VERSION: u32 = 0x01;

    /// Convert YCoCgVariant to its packed representation
    fn variant_to_u32(variant: YCoCgVariant) -> u32 {
        match variant {
            YCoCgVariant::Variant1 => 0,
            YCoCgVariant::Variant2 => 1,
            YCoCgVariant::Variant3 => 2,
            YCoCgVariant::None => 3,
        }
    }

    /// Convert packed representation back to YCoCgVariant
    fn u32_to_variant(value: u32) -> Result<YCoCgVariant, EmbedError> {
        match value {
            0 => Ok(YCoCgVariant::Variant1),
            1 => Ok(YCoCgVariant::Variant2),
            2 => Ok(YCoCgVariant::Variant3),
            3 => Ok(YCoCgVariant::None),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    /// Create BC1HeaderData from Bc1DetransformDetails
    pub fn from_detransform_details(details: Bc1DetransformDetails) -> Self {
        let mut header = Self::default();
        header.set_header_version(Self::BC1_V1_HEADER_VERSION);
        header.set_split_colour_endpoints(details.split_colour_endpoints);
        header.set_decorrelation_variant(Self::variant_to_u32(details.decorrelation_mode));
        header.set_reserved(0);
        header
    }

    /// Convert BC1HeaderData to Bc1DetransformDetails
    pub fn to_detransform_details(self) -> Result<Bc1DetransformDetails, EmbedError> {
        if self.header_version() != Self::BC1_V1_HEADER_VERSION {
            return Err(EmbedError::CorruptedEmbeddedData);
        }

        let decorrelation_mode = Self::u32_to_variant(self.decorrelation_variant())?;

        Ok(Bc1DetransformDetails {
            decorrelation_mode,
            split_colour_endpoints: self.split_colour_endpoints(),
        })
    }
}

/// Validates that the given pointer and length represent a valid DDS file.
///
/// For files that may have embedded data, this function temporarily restores the magic header
/// to perform validation.
///
/// # Safety
///
/// - `ptr` must be valid for reads of `len` bytes
///
/// # Parameters
///
/// - `ptr`: Pointer to the potential DDS data
/// - `len`: Length of the data in bytes
///
/// # Returns
///
/// - `Ok(())` if the data represents a valid DDS file
/// - `Err(EmbedError)` if validation fails
unsafe fn validate_dds_for_embedding(ptr: *const u8, len: usize) -> Result<(), EmbedError> {
    use crate::dds::constants::DDS_MAGIC;

    // First try direct validation
    if is_dds(ptr, len) {
        return Ok(());
    }

    // If that fails, it might be a DDS file with embedded data
    // Temporarily restore the magic header and check again
    let magic_ptr = ptr as *const u32;
    let _current_magic = magic_ptr.read_unaligned();

    // Check if this looks like it might have embedded data
    // We can't easily detect this without knowing the header format, so just check if
    // restoring with DDS magic makes it valid
    {
        // Currently only BC1 v1 supported
        let mut temp_data = vec![0u8; core::cmp::min(len, 128)];
        core::ptr::copy_nonoverlapping(ptr, temp_data.as_mut_ptr(), temp_data.len());

        // Try restoring with DDS magic
        (temp_data.as_mut_ptr() as *mut u32).write_unaligned(DDS_MAGIC);

        if is_dds(temp_data.as_ptr(), temp_data.len()) {
            return Ok(());
        }
    }

    Err(EmbedError::NotADds)
}

/// Embeds BC1 transform details into the DDS magic header.
///
/// # Safety
///
/// - `ptr` must be valid for reads and writes of `len` bytes
/// - `ptr` must point to a valid DDS file with BC1 format
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data (will be modified)
/// - `len`: Length of the DDS data
/// - `details`: BC1 transform details to embed
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(EmbedError)` on failure
pub unsafe fn embed_bc1_details(
    ptr: *mut u8,
    len: usize,
    details: Bc1DetransformDetails,
) -> Result<(), EmbedError> {
    validate_dds_for_embedding(ptr, len)?;

    let header_data = Bc1HeaderData::from_detransform_details(details);

    // Replace the entire 4-byte magic header with our packed data
    let magic_ptr = ptr as *mut u32;
    magic_ptr.write_unaligned(header_data.0);

    Ok(())
}

/// Unembeds BC1 transform details from the DDS magic header.
///
/// # Safety
///
/// - `ptr` must be valid for reads of `len` bytes
/// - The DDS file must have been previously embedded with BC1 details
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data with embedded details
/// - `len`: Length of the DDS data
///
/// # Returns
///
/// - `Ok(Bc1DetransformDetails)` on success
/// - `Err(EmbedError)` on failure
pub unsafe fn unembed_bc1_details(
    ptr: *const u8,
    len: usize,
) -> Result<Bc1DetransformDetails, EmbedError> {
    validate_dds_for_embedding(ptr, len)?;

    let magic_ptr = ptr as *const u32;
    let packed_data = magic_ptr.read_unaligned();

    let header_data = Bc1HeaderData(packed_data);
    header_data.to_detransform_details()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_MAGIC;

    // Helper to create a minimal DDS header for testing
    fn create_test_dds_data() -> Vec<u8> {
        let mut data = vec![0u8; 128]; // Minimum DDS header size
        unsafe {
            (data.as_mut_ptr() as *mut u32).write_unaligned(DDS_MAGIC);
        }
        data
    }

    #[test]
    fn test_bc1_header_data_bitfield() {
        let details = Bc1DetransformDetails {
            decorrelation_mode: YCoCgVariant::Variant3,
            split_colour_endpoints: false,
        };

        let header = Bc1HeaderData::from_detransform_details(details);
        assert_eq!(header.header_version(), 0x01u32); // BC1 v1
        assert!(!header.split_colour_endpoints());
        assert_eq!(header.decorrelation_variant(), 2); // Variant3 = 2
        assert_eq!(header.reserved(), 0);

        let recovered = header.to_detransform_details().unwrap();
        assert_eq!(details, recovered);
    }

    #[test]
    fn test_bc1_embed_unembed_roundtrip() {
        let mut data = create_test_dds_data();

        let original_details = Bc1DetransformDetails {
            decorrelation_mode: YCoCgVariant::Variant2,
            split_colour_endpoints: true,
        };

        unsafe {
            // Embed details
            embed_bc1_details(data.as_mut_ptr(), data.len(), original_details).unwrap();

            // Unembed details
            let recovered_details = unembed_bc1_details(data.as_ptr(), data.len()).unwrap();

            // Verify roundtrip
            assert_eq!(original_details, recovered_details);
        }
    }

    #[test]
    fn test_all_bc1_variant_combinations() {
        let mut data = create_test_dds_data();

        let variants = [
            YCoCgVariant::Variant1,
            YCoCgVariant::Variant2,
            YCoCgVariant::Variant3,
            YCoCgVariant::None,
        ];

        for &variant in &variants {
            for &split in &[true, false] {
                let details = Bc1DetransformDetails {
                    decorrelation_mode: variant,
                    split_colour_endpoints: split,
                };

                unsafe {
                    embed_bc1_details(data.as_mut_ptr(), data.len(), details).unwrap();
                    let recovered = unembed_bc1_details(data.as_ptr(), data.len()).unwrap();
                    assert_eq!(
                        details, recovered,
                        "Failed for variant {variant:?} split {split}",
                    );
                }
            }
        }
    }

    #[test]
    fn test_header_version_validation() {
        let mut data = create_test_dds_data();

        unsafe {
            // Create invalid header version
            let invalid_header = Bc1HeaderData(0xFF); // Wrong version
            let magic_ptr = data.as_mut_ptr() as *mut u32;
            magic_ptr.write_unaligned(invalid_header.0);

            // Should fail to unembed due to wrong version
            assert_eq!(
                unembed_bc1_details(data.as_ptr(), data.len()),
                Err(EmbedError::CorruptedEmbeddedData)
            );
        }
    }
}

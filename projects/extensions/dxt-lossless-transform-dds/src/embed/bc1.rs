//! BC1 format embedding support.

use dxt_lossless_transform_api_common::embed::{
    EmbedError, EmbeddableTransformDetails, TransformHeader,
};
use dxt_lossless_transform_bc1::Bc1DetransformDetails;
use dxt_lossless_transform_bc1_api::embed::EmbeddableBc1Details;

/// Embeds BC1 transform details into the DDS magic header.
///
/// # Safety
///
/// - `ptr` must be valid for reads and writes of 4 bytes
/// - `ptr` must point to a valid DDS file with BC1 format
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data (will be modified)
/// - `details`: BC1 transform details to embed
///
/// # Returns
///
/// - `Ok(())` on success
/// - [`EmbedError`] on failure
pub unsafe fn embed_bc1_details(ptr: *mut u8, details: Bc1DetransformDetails) {
    let embeddable = EmbeddableBc1Details::from(details);
    let header = embeddable.to_header();
    header.write_to_ptr(ptr);
}

/// Unembeds BC1 transform details from the DDS magic header.
///
/// # Safety
///
/// - `ptr` must be valid (non-null) for read of 4 bytes
/// - The DDS file must have been previously embedded with BC1 details
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data with embedded details
///
/// # Returns
///
/// - `Ok(Bc1DetransformDetails)` on success
/// - [`EmbedError`] on failure
pub unsafe fn unembed_bc1_details(ptr: *const u8) -> Result<Bc1DetransformDetails, EmbedError> {
    let header = TransformHeader::read_from_ptr(ptr);
    let embeddable = EmbeddableBc1Details::from_header(header)?;
    Ok(embeddable.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_MAGIC;
    use dxt_lossless_transform_api_common::embed::TransformFormat;
    use dxt_lossless_transform_common::color_565::YCoCgVariant;

    // Helper to create a minimal DDS header for testing
    fn create_test_dds_data() -> Vec<u8> {
        let mut data = vec![0u8; 128]; // Minimum DDS header size
        unsafe {
            (data.as_mut_ptr() as *mut u32).write_unaligned(DDS_MAGIC);
        }
        data
    }

    #[test]
    fn test_roundtrip_all_possible_transform_details() {
        let mut data = create_test_dds_data();

        for &variant in YCoCgVariant::all_values() {
            for &split in &[true, false] {
                let details = Bc1DetransformDetails {
                    decorrelation_mode: variant,
                    split_colour_endpoints: split,
                };

                unsafe {
                    embed_bc1_details(data.as_mut_ptr(), details);
                    let recovered = unembed_bc1_details(data.as_ptr()).unwrap();
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
            // Create invalid header with wrong format
            let header = TransformHeader::new(TransformFormat::Bc7, 0xFF);
            header.write_to_ptr(data.as_mut_ptr());

            // Should fail to unembed due to wrong format
            assert!(matches!(
                unembed_bc1_details(data.as_ptr()),
                Err(EmbedError::InvalidHeaderFormat)
            ));
        }
    }
}

//! BC1 format embedding support.

use crate::dds::constants::DDS_MAGIC;
use dxt_lossless_transform_api_common::embed::{
    EmbedError, EmbeddableTransformDetails, TransformHeader,
};
use dxt_lossless_transform_bc1::Bc1DetransformDetails;
use dxt_lossless_transform_bc1_api::embed::EmbeddableBc1Details;

/// Embeds BC1 transform details into the DDS magic header.
///
/// This function **overwrites** the original DDS magic header with transform details.
/// The original header can be restored using [`unembed_bc1_details`].
///
/// # Safety
///
/// - `ptr` must be valid for reads and writes of 4 bytes
/// - `ptr` must point to a valid DDS file with BC1 format
/// - **The caller must verify the DDS magic header is present** using [`is_dds`] before calling this function
/// - Do not rely solely on file extensions - always validate the actual header content
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data (the first 4 bytes will be overwritten)
/// - `details`: BC1 transform details to embed
///
/// # Remarks
///
/// The DDS magic header (typically `DDS_MAGIC`) is temporarily replaced with
/// embedded transform details. This allows the transform information to be
/// stored within the file without increasing file size.
///
/// **Important**: This function assumes the file has already been validated as a proper
/// DDS file. Use [`is_dds`] to verify the magic header before embedding.
///
/// [`is_dds`]: crate::dds::is_dds
pub unsafe fn embed_bc1_details(ptr: *mut u8, details: Bc1DetransformDetails) {
    let embeddable = EmbeddableBc1Details::from(details);
    let header = embeddable.to_header();
    header.write_to_ptr(ptr);
}

/// Unembeds BC1 transform details from the DDS magic header and restores the original header.
///
/// This function reads the transform details from the DDS header and then **restores**
/// the original DDS magic header, returning the file to its original state.
///
/// # Safety
///
/// - `ptr` must be valid for reads and writes of 4 bytes
/// - The DDS file must have been previously embedded with BC1 details using [`embed_bc1_details`]
/// - **The caller must ensure the data contains valid embedded transform details**
/// - Do not call this on arbitrary files - only on files that were previously embedded
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data with embedded details (the first 4 bytes will be restored)
///
/// # Returns
///
/// - `Ok(Bc1DetransformDetails)` on success
/// - [`EmbedError`] on failure
///
/// # Remarks
///
/// After successful unembedding, the DDS file header is restored to its original
/// state with the proper `DDS_MAGIC` value, making the file a valid DDS file again.
///
/// **Important**: This function will attempt to restore the DDS magic header regardless
/// of whether the input data is valid. Only call this on files that were previously
/// processed with [`embed_bc1_details`].
pub unsafe fn unembed_bc1_details(ptr: *mut u8) -> Result<Bc1DetransformDetails, EmbedError> {
    let header = TransformHeader::read_from_ptr(ptr);
    let embeddable = EmbeddableBc1Details::from_header(header)?;

    // Restore the original DDS magic header
    (ptr as *mut u32).write_unaligned(DDS_MAGIC);

    Ok(embeddable.into())
}

#[cfg(test)]
mod tests {
    use super::*;
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
                    let recovered = unembed_bc1_details(data.as_mut_ptr()).unwrap();
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
                unembed_bc1_details(data.as_mut_ptr()),
                Err(EmbedError::InvalidHeaderFormat)
            ));
        }
    }

    #[test]
    fn test_header_restoration() {
        let mut data = create_test_dds_data();

        let details = Bc1DetransformDetails {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        unsafe {
            // Verify original header is DDS_MAGIC
            let original_header = (data.as_ptr() as *const u32).read_unaligned();
            assert_eq!(original_header, DDS_MAGIC);

            // Embed details (should overwrite header)
            embed_bc1_details(data.as_mut_ptr(), details);

            // Verify header has changed
            let embedded_header = (data.as_ptr() as *const u32).read_unaligned();
            assert_ne!(embedded_header, DDS_MAGIC);

            // Unembed details (should restore original header)
            let recovered = unembed_bc1_details(data.as_mut_ptr()).unwrap();
            assert_eq!(details, recovered);

            // Verify header is restored to DDS_MAGIC
            let restored_header = (data.as_ptr() as *const u32).read_unaligned();
            assert_eq!(restored_header, DDS_MAGIC);
        }
    }
}

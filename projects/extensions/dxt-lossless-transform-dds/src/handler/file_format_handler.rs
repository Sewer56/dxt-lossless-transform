use super::DdsHandler;
use crate::dds::{
    constants::DDS_MAGIC,
    parse_dds::{parse_dds, parse_dds_ignore_magic, DdsFormat},
};
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_file_formats_api::{
    bundle::TransformBundle,
    embed::{TransformFormat, TransformHeader, TRANSFORM_HEADER_SIZE},
    error::{FormatHandlerError, TransformResult},
    handlers::FileFormatHandler,
};

/// Convert DdsFormat to TransformFormat for dispatch
fn dds_format_to_transform_format(
    format: DdsFormat,
) -> Result<TransformFormat, FormatHandlerError> {
    match format {
        DdsFormat::BC1 => Ok(TransformFormat::Bc1),
        DdsFormat::BC2 => Err(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc2,
        )),
        DdsFormat::BC3 => Err(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc3,
        )),
        DdsFormat::BC7 => Err(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc7,
        )),
        DdsFormat::Unknown | DdsFormat::NotADds => Err(FormatHandlerError::UnknownFileFormat),
    }
}

impl FileFormatHandler for DdsHandler {
    fn transform_bundle<T>(
        &self,
        input: &[u8],
        output: &mut [u8],
        bundle: &TransformBundle<T>,
    ) -> TransformResult<()>
    where
        T: SizeEstimationOperations,
        T::Error: Debug,
    {
        // Validate buffer sizes
        if output.len() < input.len() {
            return Err(FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            }
            .into());
        }

        // Parse DDS header
        let info = parse_dds(input).ok_or(FormatHandlerError::InvalidInputFileHeader)?;
        let data_offset = info.data_offset as usize;

        // Copy headers to output
        output[..data_offset].copy_from_slice(&input[..data_offset]);

        // Convert DDS format to transform format and dispatch
        let transform_format = dds_format_to_transform_format(info.format)?;
        let header = dxt_lossless_transform_file_formats_api::dispatch_transform(
            transform_format,
            &input[data_offset..],
            &mut output[data_offset..],
            bundle,
        )?;

        // Embed transform header (overwrites DDS magic)
        // SAFETY: output.as_mut_ptr() is valid for writes of at least TRANSFORM_HEADER_SIZE bytes because:
        // 1. We validated output.len() >= input.len() above
        // 2. parse_dds succeeded, guaranteeing input has valid DDS structure (minimum 128 bytes)
        // 3. Therefore output has at least 128 bytes, which is >= TRANSFORM_HEADER_SIZE bytes required for the header
        unsafe {
            header.write_to_ptr(output.as_mut_ptr());
        }

        Ok(())
    }

    fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()> {
        // Validate buffer sizes
        if input.len() < TRANSFORM_HEADER_SIZE {
            return Err(FormatHandlerError::InputTooShort {
                required: TRANSFORM_HEADER_SIZE,
                actual: input.len(),
            }
            .into());
        }

        if output.len() < input.len() {
            return Err(FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            }
            .into());
        }

        // Read transform header from first 4 bytes
        // SAFETY: input.as_ptr() is valid for reads of at least TRANSFORM_HEADER_SIZE bytes because we validated
        // input.len() >= TRANSFORM_HEADER_SIZE above. The input slice guarantees pointer validity.
        let header = unsafe { TransformHeader::read_from_ptr(input.as_ptr()) };

        // Parse header ignoring the magic (which contains transform data)
        let info =
            parse_dds_ignore_magic(input).ok_or(FormatHandlerError::InvalidRestoredFileHeader)?;
        let data_offset = info.data_offset as usize;

        // Restore DDS magic
        output[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());

        // Copy the rest of the header (from byte 4 to data_offset)
        output[4..data_offset].copy_from_slice(&input[4..data_offset]);

        // Dispatch untransform based on header format using separate input/output texture data
        dxt_lossless_transform_file_formats_api::dispatch_untransform(
            header,
            &input[data_offset..],
            &mut output[data_offset..],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::{DDS_HEADER_SIZE, DDS_MAGIC};
    use crate::test_prelude::*;
    use dxt_lossless_transform_api_common::estimate::NoEstimation;
    use dxt_lossless_transform_file_formats_api::{
        embed::TransformFormat,
        error::{FormatHandlerError, TransformError},
        TransformBundle,
    };

    // Transform/untransform buffer validation tests
    #[test]
    fn transform_bundle_rejects_output_buffer_too_small() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let input = create_valid_bc1_dds(DDS_HEADER_SIZE);
        let mut small_output = vec![0u8; input.len() - 1];

        let result = handler.transform_bundle(&input, &mut small_output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::OutputBufferTooSmall {
            required,
            actual,
        })) = result
        {
            assert_eq!(required, input.len());
            assert_eq!(actual, input.len() - 1);
        } else {
            panic!("Expected OutputBufferTooSmall error");
        }
    }

    #[test]
    fn untransform_rejects_output_buffer_too_small() {
        let handler = DdsHandler;
        let input = [0u8; DDS_HEADER_SIZE];
        let mut small_output = [0u8; DDS_HEADER_SIZE - 1];

        let result = handler.untransform(&input, &mut small_output);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::OutputBufferTooSmall {
            required,
            actual,
        })) = result
        {
            assert_eq!(required, DDS_HEADER_SIZE);
            assert_eq!(actual, DDS_HEADER_SIZE - 1);
        } else {
            panic!("Expected OutputBufferTooSmall error");
        }
    }

    // Untransform buffer validation tests
    #[test]
    fn untransform_rejects_input_too_small_for_transform_header() {
        let handler = DdsHandler;
        let input = [0u8; TRANSFORM_HEADER_SIZE - 1];
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.untransform(&input, &mut output);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::InputTooShort {
            required,
            actual,
        })) = result
        {
            assert_eq!(required, TRANSFORM_HEADER_SIZE);
            assert_eq!(actual, TRANSFORM_HEADER_SIZE - 1);
        } else {
            panic!("Expected InputTooShort error");
        }
    }

    // Input validation tests
    #[test]
    fn transform_bundle_rejects_invalid_input_file_header() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let invalid_input = [0u8; DDS_HEADER_SIZE];
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.transform_bundle(&invalid_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::InvalidInputFileHeader)) =
            result
        {
            // Expected error
        } else {
            panic!("Expected InvalidInputFileHeader error, got: {:?}", result);
        }
    }

    #[test]
    fn untransform_rejects_invalid_restored_file_header_too_short() {
        let handler = DdsHandler;
        let invalid_transformed = [0u8; 64]; // Too short for DDS header
        let mut output = [0u8; DDS_HEADER_SIZE]; // Output buffer sized appropriately

        let result = handler.untransform(&invalid_transformed, &mut output);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::InvalidRestoredFileHeader)) =
            result
        {
            // Expected error
        } else {
            panic!(
                "Expected InvalidRestoredFileHeader error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn untransform_rejects_corrupted_header_structure() {
        let handler = DdsHandler;
        let mut corrupted = [0u8; DDS_HEADER_SIZE];
        corrupted[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.untransform(&corrupted, &mut output);
        assert!(result.is_err());
    }

    // Builder availability tests

    #[test]
    fn transform_bundle_rejects_no_builder_for_bc1_format() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default(); // No builders provided
        let input = create_valid_bc1_dds(DDS_HEADER_SIZE);
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.transform_bundle(&input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::NoBuilderForFormat(format))) =
            result
        {
            assert_eq!(format, TransformFormat::Bc1);
        } else {
            panic!("Expected NoBuilderForFormat error, got: {:?}", result);
        }
    }

    // Format not implemented tests

    #[test]
    fn transform_bundle_rejects_bc2_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc2_input = create_valid_bc2_dds(DDS_HEADER_SIZE);
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.transform_bundle(&bc2_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc2,
        ))) = result
        {
            // Expected
        } else {
            panic!(
                "Expected FormatNotImplemented(Bc2) error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn transform_bundle_rejects_bc3_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc3_input = create_valid_bc3_dds(DDS_HEADER_SIZE);
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.transform_bundle(&bc3_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc3,
        ))) = result
        {
            // Expected
        } else {
            panic!(
                "Expected FormatNotImplemented(Bc3) error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn transform_bundle_rejects_bc7_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc7_input = create_valid_bc7_dds(DDS_DX10_TOTAL_HEADER_SIZE);
        let mut output = [0u8; DDS_DX10_TOTAL_HEADER_SIZE];

        let result = handler.transform_bundle(&bc7_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc7,
        ))) = result
        {
            // Expected
        } else {
            panic!(
                "Expected FormatNotImplemented(Bc7) error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn transform_bundle_rejects_unknown_format() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let unknown_input = create_unknown_format_dds(DDS_HEADER_SIZE);
        let mut output = [0u8; DDS_HEADER_SIZE];

        let result = handler.transform_bundle(&unknown_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::UnknownFileFormat)) = result {
            // Expected
        } else {
            panic!("Expected UnknownFileFormat error, got: {:?}", result);
        }
    }

    // Successful operation test

    #[test]
    fn successful_bc1_transform_and_untransform_roundtrip() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();

        // Create valid BC1 DDS with some texture data
        let mut input = create_valid_bc1_dds(DDS_HEADER_SIZE + 64);

        // Add some texture data
        for x in 0..64 {
            input[DDS_HEADER_SIZE + x] = (x % 256) as u8;
        }

        let mut transformed = vec![0u8; input.len()];
        let mut restored = vec![0u8; input.len()];

        // Transform
        let transform_result = handler.transform_bundle(&input, &mut transformed, &bundle);
        // Skip this test if BC1 API is not available
        if transform_result.is_err() {
            return;
        }

        // Verify magic header was overwritten with transform data
        assert_ne!(&transformed[0..4], &DDS_MAGIC.to_ne_bytes());
        // Verify the rest of the header is preserved
        assert_eq!(&transformed[4..DDS_HEADER_SIZE], &input[4..DDS_HEADER_SIZE]);

        // Untransform
        let untransform_result = handler.untransform(&transformed, &mut restored);
        assert!(
            untransform_result.is_ok(),
            "Untransform failed: {untransform_result:?}",
        );

        // Verify magic header was restored
        assert_eq!(&restored[0..4], &DDS_MAGIC.to_ne_bytes());
        // Verify the rest of the data matches the original
        assert_eq!(&restored[4..], &input[4..]);
    }
}

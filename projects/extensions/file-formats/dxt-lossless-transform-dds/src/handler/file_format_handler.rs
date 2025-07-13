use super::{format_conversion::dds_format_to_transform_format, DdsHandler};
use crate::dds::{
    constants::DDS_MAGIC,
    parse_dds::{parse_dds, parse_dds_ignore_magic},
};
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_file_formats_api::{
    bundle::TransformBundle,
    embed::{TransformHeader, TRANSFORM_HEADER_SIZE},
    error::{FormatHandlerError, TransformResult},
    handlers::FileFormatHandler,
};

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
        let data_length = info.data_length as usize;
        let total_required = data_offset + data_length;

        // Validate input buffer contains enough data for declared texture size
        if input.len() < total_required {
            return Err(FormatHandlerError::InputTooShortForStatedTextureSize {
                required: total_required,
                actual: input.len(),
            }
            .into());
        }

        // Copy headers to output
        output[..data_offset].copy_from_slice(&input[..data_offset]);

        // Convert DDS format to transform format and dispatch (only texture data)
        let transform_format = dds_format_to_transform_format(info.format, false)?;
        let header = dxt_lossless_transform_file_formats_api::dispatch_transform(
            transform_format,
            &input[data_offset..data_offset + data_length],
            &mut output[data_offset..data_offset + data_length],
            bundle,
        )?;

        // Copy leftover data after texture data verbatim
        let leftover_start = data_offset + data_length;
        if input.len() > leftover_start {
            output[leftover_start..].copy_from_slice(&input[leftover_start..]);
        }

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
        let data_length = info.data_length as usize;
        let total_required = data_offset + data_length;

        // Validate input buffer contains enough data for declared texture size
        if input.len() < total_required {
            return Err(FormatHandlerError::InputTooShortForStatedTextureSize {
                required: total_required,
                actual: input.len(),
            }
            .into());
        }

        // Restore DDS magic
        output[0..4].copy_from_slice(&DDS_MAGIC.to_le_bytes());

        // Copy the rest of the header (from byte 4 to data_offset)
        output[4..data_offset].copy_from_slice(&input[4..data_offset]);

        // Dispatch untransform based on header format (only texture data)
        dxt_lossless_transform_file_formats_api::dispatch_untransform(
            header,
            &input[data_offset..data_offset + data_length],
            &mut output[data_offset..data_offset + data_length],
        )?;

        // Copy leftover data after texture data verbatim
        let leftover_start = data_offset + data_length;
        if input.len() > leftover_start {
            output[leftover_start..].copy_from_slice(&input[leftover_start..]);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_HEADER_SIZE;
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
        let input = create_incomplete_bc1_dds(); // Header-only DDS for testing buffer validation
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
        let input = create_valid_bc1_dds_with_dimensions(64, 64, 1); // 64x64 BC1 texture
        let mut output = vec![0u8; input.len()];

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

    #[test]
    fn transform_bundle_accepts_bc2_format() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc2_input = create_valid_bc2_dds(); // Valid BC2 DDS for testing format support
        let mut output = vec![0u8; bc2_input.len()];

        let result = handler.transform_bundle(&bc2_input, &mut output, &bundle);
        assert!(result.is_ok(), "BC2 transform should succeed: {:?}", result);
    }

    // Format not implemented tests

    #[test]
    fn transform_bundle_rejects_bc3_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc3_input = create_valid_bc3_dds(); // Valid BC3 DDS for testing format not implemented
        let mut output = vec![0u8; bc3_input.len()];

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
    fn transform_bundle_rejects_bc6h_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc6h_input = create_valid_bc6h_dds(); // Valid BC6H DDS for testing format not implemented
        let mut output = vec![0u8; bc6h_input.len()];

        let result = handler.transform_bundle(&bc6h_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
            TransformFormat::Bc6H,
        ))) = result
        {
            // Expected
        } else {
            panic!(
                "Expected FormatNotImplemented(Bc6H) error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn transform_bundle_rejects_bc7_format_not_implemented() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();
        let bc7_input = create_valid_bc7_dds(); // Valid BC7 DDS for testing format not implemented
        let mut output = vec![0u8; bc7_input.len()];

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
        let unknown_input = create_valid_unknown_format_dds(); // Valid unknown format DDS for testing
        let mut output = vec![0u8; unknown_input.len()];

        let result = handler.transform_bundle(&unknown_input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(FormatHandlerError::UnknownFileFormat)) = result {
            // Expected
        } else {
            panic!("Expected UnknownFileFormat error, got: {:?}", result);
        }
    }

    // Data length and leftover data tests
    #[test]
    fn transform_and_untransform_preserves_leftover_data_roundtrip() {
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let leftover_data = b"Roundtrip preservation test data 123456!";
        let input = create_bc1_dds_with_leftover_data(32, 32, leftover_data);
        let mut transformed = vec![0u8; input.len()];
        let mut restored = vec![0u8; input.len()];

        // Transform
        let transform_result = handler.transform_bundle(&input, &mut transformed, &bundle);
        if transform_result.is_err() {
            return; // Skip test if BC1 not available
        }

        // Untransform
        let untransform_result = handler.untransform(&transformed, &mut restored);
        assert!(
            untransform_result.is_ok(),
            "Untransform failed: {untransform_result:?}"
        );

        // Verify complete roundtrip preserves all data including leftover
        assert_eq!(restored, input);
    }

    #[test]
    fn transform_bundle_rejects_insufficient_data_for_declared_size() {
        // size as in width+height
        let handler = DdsHandler;
        let bundle = TransformBundle::<NoEstimation>::default_all();

        // Create DDS that declares larger data size than available
        let mut input = create_valid_bc1_dds_with_dimensions(64, 64, 1);
        // Truncate the data to simulate insufficient buffer
        input.truncate(input.len() - 100);
        let mut output = vec![0u8; input.len()];

        let result = handler.transform_bundle(&input, &mut output, &bundle);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(
            FormatHandlerError::InputTooShortForStatedTextureSize { .. },
        )) = result
        {
            // Expected error type
        } else {
            panic!(
                "Expected InputTooShortForStatedTextureSize error, got: {:?}",
                result
            );
        }
    }

    #[test]
    fn untransform_rejects_insufficient_data_for_declared_size() {
        // size as in width+height
        let handler = DdsHandler;

        // Create a simulated transform that would require more data than provided
        let mut transformed_data = create_valid_bc1_dds_with_dimensions(64, 64, 1);
        // Overwrite magic with transform header (simulating transformed file)
        transformed_data[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        // Truncate to simulate insufficient data
        transformed_data.truncate(transformed_data.len() - 50);

        let mut output = vec![0u8; transformed_data.len()];

        let result = handler.untransform(&transformed_data, &mut output);
        assert!(result.is_err());

        if let Err(TransformError::FormatHandler(
            FormatHandlerError::InputTooShortForStatedTextureSize { .. },
        )) = result
        {
            // Expected error type
        } else {
            panic!(
                "Expected InputTooShortForStatedTextureSize error, got: {:?}",
                result
            );
        }
    }
}

//! Slice-to-file transformation operations using memory mapping.

use crate::bundle::TransformBundle;
use crate::file_io::FileOperationResult;
use crate::handlers::{FileFormatDetection, FileFormatHandler, FileFormatUntransformDetection};
use crate::TransformError;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::path::Path;

/// Transform a slice using a specific handler and write to a file.
///
/// This function takes an input slice, transforms it using the provided handler
/// and bundle, and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_data` - Input data slice to transform
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `bundle` - The transform bundle containing BCx builders
///
/// # Returns
///
/// Result indicating success or error
pub fn transform_slice_to_file_with_handler<H: FileFormatHandler, T>(
    handler: &H,
    input_data: &[u8],
    output_path: &Path,
    bundle: &TransformBundle<T>,
) -> FileOperationResult<()>
where
    T: SizeEstimationOperations,
    T::Error: Debug,
{
    let input_size = input_data.len();
    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
    let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Transform directly into the memory-mapped output
    crate::api::transform_slice_with_bundle(
        handler,
        input_data,
        output_mapping.as_mut_slice(),
        bundle,
    )?;

    Ok(())
}

/// Untransform a slice using a specific handler and write to a file.
///
/// This function takes an input slice, untransforms it using the provided handler,
/// and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_data` - Input data slice to untransform
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
///
/// # Returns
///
/// Result indicating success or error
pub fn untransform_slice_to_file_with_handler<H: FileFormatHandler>(
    handler: &H,
    input_data: &[u8],
    output_path: &Path,
) -> FileOperationResult<()> {
    let input_size = input_data.len();
    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
    let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Untransform directly into the memory-mapped output
    crate::api::untransform_slice(handler, input_data, output_mapping.as_mut_slice())?;

    Ok(())
}

/// Transform a slice using multiple handlers with automatic format detection and write to a file.
///
/// This function tries each handler in sequence until one accepts the slice format,
/// then transforms the slice using that handler and the provided bundle.
///
/// # Arguments
///
/// * `handlers` - Iterator of file format handlers that implement [`FileFormatDetection`]
/// * `input_data` - Input data slice to transform
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `bundle` - The transform bundle containing BCx builders
/// * `file_extension` - Optional file extension hint for format detection
///
/// # Returns
///
/// Result indicating success or error. Returns [`crate::error::TransformError::NoSupportedHandler`]
/// if no handler can process the slice.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{
///     file_io::{transform_slice_to_file_with_multiple_handlers, FileOperationResult},
///     TransformBundle
/// };
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// use std::path::Path;
///
/// fn example_transform_slice_multiple_handlers(
///     input_data: &[u8],
///     output_path: &Path
/// ) -> FileOperationResult<()> {
///     let handlers = [DdsHandler];
///     let bundle = TransformBundle::<NoEstimation>::default_all();
///     transform_slice_to_file_with_multiple_handlers(
///         handlers,
///         input_data,
///         output_path,
///         &bundle,
///         Some("dds")
///     )?;
///     Ok(())
/// }
/// ```
pub fn transform_slice_to_file_with_multiple_handlers<HandlerIterator, Handler, SizeEstimator>(
    handlers: HandlerIterator,
    input_data: &[u8],
    output_path: &Path,
    bundle: &TransformBundle<SizeEstimator>,
    file_extension: Option<&str>,
) -> FileOperationResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatDetection,
    SizeEstimator: SizeEstimationOperations,
    SizeEstimator::Error: Debug,
{
    // Try each handler until one accepts the slice
    for handler in handlers {
        if handler.can_handle(input_data, file_extension) {
            let input_size = input_data.len();
            let output_handle =
                ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
            let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

            // Transform using the accepting handler
            return crate::api::transform_slice_with_bundle(
                &handler,
                input_data,
                output_mapping.as_mut_slice(),
                bundle,
            )
            .map_err(Into::into);
        }
    }

    // No handler could process the slice
    Err(TransformError::NoSupportedHandler.into())
}

/// Untransform a slice using multiple handlers with automatic format detection and write to a file.
///
/// This function tries each handler in sequence until one accepts the slice format,
/// then untransforms the slice using that handler.
///
/// # Arguments
///
/// * `handlers` - Iterator of file format handlers that implement [`FileFormatUntransformDetection`]
/// * `input_data` - Input data slice to untransform
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `file_extension` - Optional file extension hint for format detection
///
/// # Returns
///
/// Result indicating success or error. Returns [`crate::error::TransformError::NoSupportedHandler`]
/// if no handler can process the slice.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::file_io::{
///     untransform_slice_to_file_with_multiple_handlers,
///     FileOperationResult
/// };
/// use dxt_lossless_transform_dds::DdsHandler;
/// use std::path::Path;
///
/// fn example_untransform_slice_multiple_handlers(
///     input_data: &[u8],
///     output_path: &Path
/// ) -> FileOperationResult<()> {
///     let handlers = [DdsHandler];
///     untransform_slice_to_file_with_multiple_handlers(
///         handlers,
///         input_data,
///         output_path,
///         Some("dds")
///     )?;
///     Ok(())
/// }
/// ```
pub fn untransform_slice_to_file_with_multiple_handlers<HandlerIterator, Handler>(
    handlers: HandlerIterator,
    input_data: &[u8],
    output_path: &Path,
    file_extension: Option<&str>,
) -> FileOperationResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatUntransformDetection,
{
    // Try each handler until one accepts the slice
    for handler in handlers {
        if handler.can_handle_untransform(input_data, file_extension) {
            let input_size = input_data.len();
            let output_handle =
                ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
            let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

            // Untransform using the accepting handler
            return crate::api::untransform_slice(
                &handler,
                input_data,
                output_mapping.as_mut_slice(),
            )
            .map_err(Into::into);
        }
    }

    // No handler could process the slice
    Err(TransformError::NoSupportedHandler.into())
}

#[cfg(test)]
mod tests {
    use super::super::test_prelude::*;
    use super::*;

    #[test]
    fn transform_slice_to_file_succeeds_with_single_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_to_file_with_handler(
            &handler,
            &input_data,
            output_file.path(),
            &bundle,
        );

        assert!(result.is_ok());
        verify_file_operation_success(output_file.path(), input_data.len());
        assert!(handler.get_calls().transform_bundle_called);
    }

    #[test]
    fn untransform_slice_to_file_succeeds_with_single_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let output_file = create_output_file();

        let result =
            untransform_slice_to_file_with_handler(&handler, &input_data, output_file.path());

        assert!(result.is_ok());
        verify_file_operation_success(output_file.path(), input_data.len());
        assert!(handler.get_calls().untransform_called);
    }

    #[test]
    fn transform_slice_to_file_with_multiple_handlers_succeeds_on_extension_match() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            &bundle,
            Some("dds"),
        );

        assert!(result.is_ok());
        verify_transform_handler_calls(&handler, Some("dds".to_string()), true);
    }

    #[test]
    fn transform_slice_to_file_with_multiple_handlers_fails_on_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            &bundle,
            Some("png"),
        );

        assert!(matches!(
            result,
            Err(crate::file_io::FileOperationError::Transform(
                TransformError::NoSupportedHandler
            ))
        ));
        verify_transform_handler_calls(&handler, Some("png".to_string()), false);
    }

    #[test]
    fn transform_slice_to_file_with_multiple_handlers_succeeds_with_no_extension() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            &bundle,
            None,
        );

        assert!(result.is_ok());
        verify_transform_handler_calls(&handler, None, true);
    }

    #[test]
    fn untransform_slice_to_file_with_multiple_handlers_succeeds_on_extension_match() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let output_file = create_output_file();

        let result = untransform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            Some("dds"),
        );

        assert!(result.is_ok());
        verify_untransform_handler_calls(&handler, Some("dds".to_string()), true);
    }

    #[test]
    fn untransform_slice_to_file_with_multiple_handlers_fails_on_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let output_file = create_output_file();

        let result = untransform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            Some("png"),
        );

        assert!(matches!(
            result,
            Err(crate::file_io::FileOperationError::Transform(
                TransformError::NoSupportedHandler
            ))
        ));
        verify_untransform_handler_calls(&handler, Some("png".to_string()), false);
    }

    #[test]
    fn transform_slice_to_file_with_multiple_handlers_succeeds_with_no_handlers_on_extensionless() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            &bundle,
            None,
        );

        assert!(result.is_ok());
        verify_transform_handler_calls(&handler, None, true);
    }

    #[test]
    fn untransform_slice_to_file_with_multiple_handlers_succeeds_with_no_handlers_on_extensionless()
    {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let output_file = create_output_file();

        let result = untransform_slice_to_file_with_multiple_handlers(
            [handler.clone()],
            &input_data,
            output_file.path(),
            None,
        );

        assert!(result.is_ok());
        verify_untransform_handler_calls(&handler, None, true);
    }
}

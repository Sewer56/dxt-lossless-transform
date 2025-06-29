//! File-to-file transformation operations using memory mapping.

use crate::bundle::TransformBundle;
use crate::file_io::FileOperationResult;
use crate::handlers::{FileFormatDetection, FileFormatHandler, FileFormatUntransformDetection};
use crate::TransformError;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::path::Path;

/// Transform a file using a specific handler and transform bundle.
///
/// This function memory-maps the input file, transforms it using the provided handler
/// and bundle, and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_path` - Path to the input file
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `bundle` - The transform bundle containing BCx builders
///
/// # Returns
///
/// Result indicating success or error
pub fn transform_file_with_handler<H: FileFormatHandler, T>(
    handler: &H,
    input_path: &Path,
    output_path: &Path,
    bundle: &TransformBundle<T>,
) -> FileOperationResult<()>
where
    T: SizeEstimationOperations,
    T::Error: Debug,
{
    // Open input file
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;
    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
    let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Transform directly into the memory-mapped output
    crate::api::transform_slice_with_bundle(
        handler,
        input_mapping.as_slice(),
        output_mapping.as_mut_slice(),
        bundle,
    )?;

    Ok(())
}

/// Untransform a file using a specific handler.
///
/// This function memory-maps the input file, untransforms it using the provided handler,
/// and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_path` - Path to the input file
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
///
/// # Returns
///
/// Result indicating success or error
pub fn untransform_file_with_handler<H: FileFormatHandler>(
    handler: &H,
    input_path: &Path,
    output_path: &Path,
) -> FileOperationResult<()> {
    // Open input file
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;

    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;
    let mut output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Untransform directly into the memory-mapped output
    crate::api::untransform_slice(
        handler,
        input_mapping.as_slice(),
        output_mapping.as_mut_slice(),
    )?;

    Ok(())
}

/// Transform a file using multiple handlers with automatic format detection.
///
/// This function tries each handler in sequence until one accepts the file format,
/// then transforms the file using that handler and the provided bundle.
///
/// # Arguments
///
/// * `handlers` - Iterator of file format handlers that implement [`FileFormatDetection`]
/// * `input_path` - Path to the input file
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `bundle` - The transform bundle containing BCx builders
///
/// # Returns
///
/// Result indicating success or error. Returns [`crate::error::TransformError::NoSupportedHandler`]
/// if no handler can process the file.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{
///     file_io::{transform_file_with_multiple_handlers, FileOperationResult},
///     TransformBundle
/// };
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// use std::path::Path;
///
/// fn example_transform_file_multiple_handlers(
///     input_path: &Path,
///     output_path: &Path
/// ) -> FileOperationResult<()> {
///     let handlers = [DdsHandler];
///     let bundle = TransformBundle::<NoEstimation>::default_all();
///     transform_file_with_multiple_handlers(handlers, input_path, output_path, &bundle)?;
///     Ok(())
/// }
/// ```
pub fn transform_file_with_multiple_handlers<HandlerIterator, Handler, SizeEstimator>(
    handlers: HandlerIterator,
    input_path: &Path,
    output_path: &Path,
    bundle: &TransformBundle<SizeEstimator>,
) -> FileOperationResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatDetection,
    SizeEstimator: SizeEstimationOperations,
    SizeEstimator::Error: Debug,
{
    // Open input file and read data for format detection
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;
    let input_data = input_mapping.as_slice();

    // Extract file extension from input path for faster format detection
    let file_extension = super::extract_lowercase_extension(input_path);
    let file_extension_ref = file_extension.as_deref();

    // Try each handler until one accepts the file
    for handler in handlers {
        if handler.can_handle(input_data, file_extension_ref) {
            // Create output file with same size as input
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

    // No handler could process the file
    Err(TransformError::NoSupportedHandler.into())
}

/// Untransform a file using multiple handlers with automatic format detection.
///
/// This function tries each handler in sequence until one accepts the transformed file format,
/// then untransforms the file using that handler.
///
/// # Arguments
///
/// * `handlers` - Iterator of file format handlers that implement [`FileFormatUntransformDetection`]
/// * `input_path` - Path to the input file (containing transformed data)
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
///
/// # Returns
///
/// Result indicating success or error. Returns [`crate::error::TransformError::NoSupportedHandler`]
/// if no handler can process the file.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::file_io::{
///     untransform_file_with_multiple_handlers,
///     FileOperationResult
/// };
/// use dxt_lossless_transform_dds::DdsHandler;
/// use std::path::Path;
///
/// fn example_untransform_file_multiple_handlers(
///     input_path: &Path,
///     output_path: &Path
/// ) -> FileOperationResult<()> {
///     let handlers = [DdsHandler];
///     untransform_file_with_multiple_handlers(handlers, input_path, output_path)?;
///     Ok(())
/// }
/// ```
pub fn untransform_file_with_multiple_handlers<HandlerIterator, Handler>(
    handlers: HandlerIterator,
    input_path: &Path,
    output_path: &Path,
) -> FileOperationResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatUntransformDetection,
{
    // Open input file and read data for format detection
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;
    let input_data = input_mapping.as_slice();

    // Extract file extension from input path for faster format detection
    let file_extension = super::extract_lowercase_extension(input_path);
    let file_extension_ref = file_extension.as_deref();

    // Try each handler until one accepts the file
    for handler in handlers {
        if handler.can_handle_untransform(input_data, file_extension_ref) {
            // Create output file with same size as input
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

    // No handler could process the file
    Err(TransformError::NoSupportedHandler.into())
}

#[cfg(test)]
mod tests {
    use super::super::test_prelude::*;
    use super::*;

    #[test]
    fn transform_file_succeeds_with_single_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        run_single_handler_test(
            &handler,
            || {
                transform_file_with_handler(
                    &handler,
                    input_file.path(),
                    output_file.path(),
                    &bundle,
                )
            },
            true,  // verify_transform_called
            false, // verify_untransform_called
        );

        verify_file_operation_success(output_file.path(), input_data.len());
    }

    #[test]
    fn untransform_file_succeeds_with_single_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();

        run_single_handler_test(
            &handler,
            || untransform_file_with_handler(&handler, input_file.path(), output_file.path()),
            false, // verify_transform_called
            true,  // verify_untransform_called
        );

        verify_file_operation_success(output_file.path(), input_data.len());
    }

    #[test]
    fn transform_with_multiple_handlers_succeeds_on_extension_match() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("dds"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        run_extension_test(
            &handler,
            || {
                transform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                    &bundle,
                )
            },
            "dds",
            ExtensionTestResult::Success,
            true, // is_transform
        );
    }

    #[test]
    fn transform_with_multiple_handlers_fails_on_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("png"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        run_extension_test(
            &handler,
            || {
                transform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                    &bundle,
                )
            },
            "png",
            ExtensionTestResult::NoSupportedHandler,
            true, // is_transform
        );
    }

    #[test]
    fn transform_with_multiple_handlers_succeeds_with_case_insensitive_extension() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("DDS"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        run_case_insensitive_extension_test(
            &handler,
            || {
                transform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                    &bundle,
                )
            },
            true, // is_transform
        );
    }

    #[test]
    fn transform_with_multiple_handlers_succeeds_with_no_extension() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        run_extensionless_test(
            &handler,
            || {
                transform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                    &bundle,
                )
            },
            true, // is_transform
        );
    }

    #[test]
    fn untransform_with_multiple_handlers_succeeds_on_extension_match() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("dds"));
        let output_file = create_output_file();

        run_extension_test(
            &handler,
            || {
                untransform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                )
            },
            "dds",
            ExtensionTestResult::Success,
            false, // is_transform
        );
    }

    #[test]
    fn untransform_with_multiple_handlers_fails_on_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("png"));
        let output_file = create_output_file();

        run_extension_test(
            &handler,
            || {
                untransform_file_with_multiple_handlers(
                    [handler.clone()],
                    input_file.path(),
                    output_file.path(),
                )
            },
            "png",
            ExtensionTestResult::NoSupportedHandler,
            false, // is_transform
        );
    }
}

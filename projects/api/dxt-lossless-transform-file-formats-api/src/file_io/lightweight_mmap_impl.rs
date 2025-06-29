//! File I/O implementation using lightweight-mmap.

use crate::bundle::TransformBundle;
use crate::file_io::FileOperationResult;
use crate::handlers::{FileFormatDetection, FileFormatHandler, FileFormatUntransformDetection};
use crate::TransformError;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::path::Path;
use std::string::String;

/// Extract file extension from a path and convert to lowercase.
///
/// # Arguments
///
/// * `path` - The file path to extract extension from
///
/// # Returns
///
/// * `Some(extension)` - The lowercase extension string without leading dot
/// * `None` - If the path has no extension
pub fn extract_lowercase_extension(path: &Path) -> Option<String> {
    // Note(sewer): Performance here is kinda oof, due to heap allocation, but this is on the
    // slow path of unknown file types; so I don't mind for the time being.
    path.extension()?.to_str().map(|s| s.to_lowercase())
}

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
    let file_extension = extract_lowercase_extension(input_path);
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
    let file_extension = extract_lowercase_extension(input_path);
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
    use super::*;
    use crate::test_prelude::*;
    use alloc::format;
    use alloc::string::{String, ToString};
    use dxt_lossless_transform_api_common::estimate::NoEstimation;
    use std::vec::Vec;
    use tempfile::{Builder, NamedTempFile};

    /// Helper function to read file contents.
    fn read_file_contents(path: &Path) -> std::io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    /// Helper to create a temporary input file with test data and optional extension.
    fn create_input_file_with_data_and_extension(
        data: &[u8],
        extension: Option<&str>,
    ) -> NamedTempFile {
        let input_file = match extension {
            Some(ext) => Builder::new()
                .suffix(&format!(".{ext}"))
                .tempfile()
                .expect("Failed to create temp file"),
            None => Builder::new()
                .prefix("test_file_")
                .tempfile()
                .expect("Failed to create temp file"),
        };

        std::fs::write(input_file.path(), data).expect("Failed to write input data");
        input_file
    }

    /// Helper to create a temporary output file.
    fn create_output_file() -> NamedTempFile {
        NamedTempFile::new().expect("Failed to create temp file")
    }

    /// Helper to verify successful file operation results.
    fn verify_file_operation_success(output_path: &Path, expected_size: usize) {
        let output_data = read_file_contents(output_path).expect("Failed to read output file");
        assert_eq!(output_data.len(), expected_size);
    }

    /// Helper to verify transform handler calls.
    fn verify_transform_handler_calls(
        handler: &MockHandler,
        expected_extension: Option<String>,
        should_have_transformed: bool,
    ) {
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_calls.len(), 1);
        assert_eq!(calls.can_handle_calls[0], expected_extension);
        assert_eq!(calls.transform_bundle_called, should_have_transformed);
    }

    /// Helper to verify untransform handler calls.
    fn verify_untransform_handler_calls(
        handler: &MockHandler,
        expected_extension: Option<String>,
        should_have_untransformed: bool,
    ) {
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_untransform_calls.len(), 1);
        assert_eq!(calls.can_handle_untransform_calls[0], expected_extension);
        assert_eq!(calls.untransform_called, should_have_untransformed);
    }

    #[test]
    fn test_transform_file_with_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result =
            transform_file_with_handler(&handler, input_file.path(), output_file.path(), &bundle);
        assert!(result.is_ok());

        verify_file_operation_success(output_file.path(), input_data.len());
        assert!(handler.get_calls().transform_bundle_called);
    }

    #[test]
    fn test_untransform_file_with_handler() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();

        let result = untransform_file_with_handler(&handler, input_file.path(), output_file.path());
        assert!(result.is_ok());

        verify_file_operation_success(output_file.path(), input_data.len());
        assert!(handler.get_calls().untransform_called);
    }

    #[test]
    fn test_transform_file_with_multiple_handlers_extension_matching() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("dds"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
            &bundle,
        );
        assert!(result.is_ok());

        verify_transform_handler_calls(&handler, Some("dds".to_string()), true);
    }

    #[test]
    fn test_transform_file_with_multiple_handlers_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("png"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
            &bundle,
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
    fn test_transform_file_with_multiple_handlers_case_insensitive_extension() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("DDS"));
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
            &bundle,
        );
        assert!(result.is_ok());

        // Extension should be converted to lowercase
        verify_transform_handler_calls(&handler, Some("dds".to_string()), true);
    }

    #[test]
    fn test_transform_file_with_multiple_handlers_no_extension() {
        let handler = MockHandler::new_extensionless_accepting();
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, None);
        let output_file = create_output_file();
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
            &bundle,
        );
        assert!(result.is_ok());

        verify_transform_handler_calls(&handler, None, true);
    }

    #[test]
    fn test_untransform_file_with_multiple_handlers_extension_matching() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("dds"));
        let output_file = create_output_file();

        let result = untransform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
        );
        assert!(result.is_ok());

        verify_untransform_handler_calls(&handler, Some("dds".to_string()), true);
    }

    #[test]
    fn test_untransform_file_with_multiple_handlers_extension_mismatch() {
        let handler = MockHandler::new_accepting("dds");
        let input_data = create_test_data(64);
        let input_file = create_input_file_with_data_and_extension(&input_data, Some("png"));
        let output_file = create_output_file();

        let result = untransform_file_with_multiple_handlers(
            [handler.clone()],
            input_file.path(),
            output_file.path(),
        );

        assert!(matches!(
            result,
            Err(crate::file_io::FileOperationError::Transform(
                TransformError::NoSupportedHandler
            ))
        ));

        verify_untransform_handler_calls(&handler, Some("png".to_string()), false);
    }
}

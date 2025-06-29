//! Test utilities for lightweight memory-mapped file operations.

use crate::file_io::FileOperationResult;
pub use crate::test_prelude::*;

// Re-export commonly used testing items to avoid repetitive imports
pub use crate::{TransformBundle, TransformError};
pub use alloc::format;
pub use alloc::string::{String, ToString};
pub use alloc::vec;
pub use dxt_lossless_transform_api_common::estimate::NoEstimation;
pub use std::vec::Vec;
pub use tempfile::{Builder, NamedTempFile};

/// Helper function to read file contents.
pub fn read_file_contents(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// Helper to create a temporary input file with test data and optional extension.
pub fn create_input_file_with_data_and_extension(
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
pub fn create_output_file() -> NamedTempFile {
    NamedTempFile::new().expect("Failed to create temp file")
}

/// Helper to verify successful file operation results.
pub fn verify_file_operation_success(output_path: &std::path::Path, expected_size: usize) {
    let output_data = read_file_contents(output_path).expect("Failed to read output file");
    assert_eq!(output_data.len(), expected_size);
}

/// Helper to verify transform handler calls.
pub fn verify_transform_handler_calls(
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
pub fn verify_untransform_handler_calls(
    handler: &MockHandler,
    expected_extension: Option<String>,
    should_have_untransformed: bool,
) {
    let calls = handler.get_calls();
    assert_eq!(calls.can_handle_untransform_calls.len(), 1);
    assert_eq!(calls.can_handle_untransform_calls[0], expected_extension);
    assert_eq!(calls.untransform_called, should_have_untransformed);
}

/// Helper to verify that output slice has expected size and basic content.
pub fn verify_slice_operation_success(output_slice: &[u8], expected_size: usize) {
    assert_eq!(output_slice.len(), expected_size);
}

/// Helper to create a test data buffer with specific size.
pub fn create_test_buffer(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

/// Test result for extension-based operations.
pub enum ExtensionTestResult {
    Success,
    NoSupportedHandler,
}

/// Generic test runner for single handler operations.
///
/// This function encapsulates the common pattern of testing operations with a single handler.
pub fn run_single_handler_test<F>(
    handler: &MockHandler,
    operation: F,
    verify_transform_called: bool,
    verify_untransform_called: bool,
) where
    F: FnOnce() -> FileOperationResult<()>,
{
    let result = operation();
    assert!(result.is_ok());

    let calls = handler.get_calls();
    assert_eq!(calls.transform_bundle_called, verify_transform_called);
    assert_eq!(calls.untransform_called, verify_untransform_called);
}

/// Generic test runner for multiple handler extension-based operations.
///
/// This function encapsulates the common pattern of testing operations with multiple handlers
/// where the test involves checking extension matching behavior.
pub fn run_extension_test<F, H>(
    handler: &MockHandler,
    operation: F,
    input_extension: &str,
    expected_result: ExtensionTestResult,
    is_transform: bool,
) where
    F: FnOnce() -> FileOperationResult<H>,
{
    let result = operation();

    match expected_result {
        ExtensionTestResult::Success => {
            assert!(result.is_ok());
            if is_transform {
                verify_transform_handler_calls(handler, Some(input_extension.to_string()), true);
            } else {
                verify_untransform_handler_calls(handler, Some(input_extension.to_string()), true);
            }
        }
        ExtensionTestResult::NoSupportedHandler => {
            assert!(matches!(
                result,
                Err(crate::file_io::FileOperationError::Transform(
                    TransformError::NoSupportedHandler
                ))
            ));
            if is_transform {
                verify_transform_handler_calls(handler, Some(input_extension.to_string()), false);
            } else {
                verify_untransform_handler_calls(handler, Some(input_extension.to_string()), false);
            }
        }
    }
}

/// Test case insensitive extension matching.
///
/// This function tests that extensions are converted to lowercase and properly matched.
pub fn run_case_insensitive_extension_test<F, H>(
    handler: &MockHandler,
    operation: F,
    is_transform: bool,
) where
    F: FnOnce() -> FileOperationResult<H>,
{
    let result = operation();
    assert!(result.is_ok());

    // Extension should be converted to lowercase
    if is_transform {
        verify_transform_handler_calls(handler, Some("dds".to_string()), true);
    } else {
        verify_untransform_handler_calls(handler, Some("dds".to_string()), true);
    }
}

/// Test extensionless file handling.
///
/// This function tests operations on files with no extension.
pub fn run_extensionless_test<F, H>(handler: &MockHandler, operation: F, is_transform: bool)
where
    F: FnOnce() -> FileOperationResult<H>,
{
    let result = operation();
    assert!(result.is_ok());

    if is_transform {
        verify_transform_handler_calls(handler, None, true);
    } else {
        verify_untransform_handler_calls(handler, None, true);
    }
}

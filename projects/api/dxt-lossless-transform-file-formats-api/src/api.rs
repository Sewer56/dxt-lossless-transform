//! High-level convenience APIs for file format operations.

use crate::bundle::TransformBundle;
use crate::error::{FormatHandlerError, TransformError, TransformResult};
use crate::handlers::{FileFormatDetection, FileFormatHandler, FileFormatUntransformDetection};
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Transform a slice using the specified format handler and transform bundle.
///
/// This is the main entry point for transforming file data. The handler will:
/// 1. Detect the specific BCx format in the file
/// 2. Use the appropriate builder from the bundle
/// 3. Embed transform details in the output header
///
/// # Parameters
///
/// - `handler`: The file format handler (e.g., DdsHandler)
/// - `input`: Input buffer containing the file data
/// - `output`: Output buffer (must be at least the same size as input)
/// - `bundle`: Bundle containing transform builders for different BCx formats
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_with_bundle};
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_transform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let bundle = TransformBundle::<NoEstimation>::default_all();
///     let mut output = vec![0u8; input.len()];
///     transform_slice_with_bundle(&DdsHandler, input, &mut output, &bundle)?;
///     Ok(output)
/// }
/// ```
pub fn transform_slice_with_bundle<H: FileFormatHandler, T>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
    bundle: &TransformBundle<T>,
) -> TransformResult<()>
where
    T: SizeEstimationOperations,
    T::Error: Debug,
{
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    handler.transform_bundle(input, output, bundle)
}

/// Untransform a slice using the specified format handler.
///
/// This will:
/// 1. Extract transform details from the header
/// 2. Restore the original file format header
/// 3. Dispatch to the appropriate untransform function
///
/// # Parameters
///
/// - `handler`: The file format handler (e.g., DdsHandler)
/// - `input`: Input buffer containing transformed data
/// - `output`: Output buffer (must be at least the same size as input)
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{untransform_slice};
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_untransform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let mut output = vec![0u8; input.len()];
///     untransform_slice(&DdsHandler, input, &mut output)?;
///     Ok(output)
/// }
/// ```
pub fn untransform_slice<H: FileFormatHandler>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
) -> TransformResult<()> {
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    handler.untransform(input, output)
}

/// Transform a slice using multiple handlers with automatic format detection.
///
/// This function tries each handler in sequence until one accepts the file format,
/// then transforms the slice using that handler and the provided bundle.
///
/// # Parameters
///
/// - `handlers`: Iterator of file format handlers that implement [`FileFormatDetection`]
/// - `input`: Input buffer containing the file data
/// - `output`: Output buffer (must be at least the same size as input)
/// - `bundle`: Bundle containing transform builders for different BCx formats
///
/// # Returns
///
/// Result indicating success or [`crate::error::TransformError::NoSupportedHandler`] if no handler can process the data.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_with_multiple_handlers, TransformResult};
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// use dxt_lossless_transform_dds::DdsHandler;
///
/// fn example_transform_multiple_handlers(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let handlers = [DdsHandler];
///     let bundle = TransformBundle::<NoEstimation>::default_all();
///     let mut output = vec![0u8; input.len()];
///     transform_slice_with_multiple_handlers(handlers, input, &mut output, &bundle)?;
///     Ok(output)
/// }
/// ```
pub fn transform_slice_with_multiple_handlers<HandlerIterator, Handler, SizeEstimator>(
    handlers: HandlerIterator,
    input: &[u8],
    output: &mut [u8],
    bundle: &TransformBundle<SizeEstimator>,
) -> TransformResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatDetection,
    SizeEstimator: SizeEstimationOperations,
    SizeEstimator::Error: Debug,
{
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    // Try each handler until one accepts the file
    for handler in handlers {
        if handler.can_handle(input, None) {
            return handler.transform_bundle(input, output, bundle);
        }
    }

    // No handler could process the file
    Err(TransformError::NoSupportedHandler)
}

/// Untransform a slice using multiple handlers with automatic format detection.
///
/// This function tries each handler in sequence until one accepts the transformed file format,
/// then untransforms the slice using that handler.
///
/// # Parameters
///
/// - `handlers`: Iterator of file format handlers that implement [`FileFormatUntransformDetection`]
/// - `input`: Input buffer containing transformed data
/// - `output`: Output buffer (must be at least the same size as input)
///
/// # Returns
///
/// Result indicating success or [`crate::error::TransformError::NoSupportedHandler`] if no handler can process the data.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{untransform_slice_with_multiple_handlers, TransformResult};
/// use dxt_lossless_transform_dds::DdsHandler;
///
/// fn example_untransform_multiple_handlers(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let handlers = [DdsHandler];
///     let mut output = vec![0u8; input.len()];
///     untransform_slice_with_multiple_handlers(handlers, input, &mut output)?;
///     Ok(output)
/// }
/// ```
pub fn untransform_slice_with_multiple_handlers<HandlerIterator, Handler>(
    handlers: HandlerIterator,
    input: &[u8],
    output: &mut [u8],
) -> TransformResult<()>
where
    HandlerIterator: IntoIterator<Item = Handler>,
    Handler: FileFormatUntransformDetection,
{
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    // Try each handler until one accepts the file
    for handler in handlers {
        if handler.can_handle_untransform(input, None) {
            return handler.untransform(input, output);
        }
    }

    // No handler could process the file
    Err(TransformError::NoSupportedHandler)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use alloc::vec;
    use dxt_lossless_transform_api_common::estimate::NoEstimation;

    #[test]
    fn test_transform_slice_with_bundle() {
        let handler = MockHandler::new_extensionless_accepting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_with_bundle(&handler, &input, &mut output, &bundle);
        assert!(result.is_ok());

        // Verify that the handler was called with correct parameters
        let calls = handler.get_calls();
        assert!(calls.transform_bundle_called);
    }

    #[test]
    fn test_untransform_slice() {
        let handler = MockHandler::new_extensionless_accepting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];

        let result = untransform_slice(&handler, &input, &mut output);
        assert!(result.is_ok());

        // Verify that the handler was called
        let calls = handler.get_calls();
        assert!(calls.untransform_called);
    }

    #[test]
    fn test_transform_slice_with_multiple_handlers_passes_none_extension() {
        let handler = MockHandler::new_accepting("dds");
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result =
            transform_slice_with_multiple_handlers([handler.clone()], &input, &mut output, &bundle);

        // Verify the handler was called with None extension (slice operations don't have file paths)
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_calls.len(), 1);
        assert_eq!(calls.can_handle_calls[0], None);

        // Handler should reject since it expects "dds" extension but gets None
        assert!(matches!(result, Err(TransformError::NoSupportedHandler)));
    }

    #[test]
    fn test_transform_slice_with_multiple_handlers_tries_all_handlers() {
        let handler1 = MockHandler::new_rejecting();
        let handler2 = MockHandler::new_extensionless_accepting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result = transform_slice_with_multiple_handlers(
            [handler1.clone(), handler2.clone()],
            &input,
            &mut output,
            &bundle,
        );
        assert!(result.is_ok());

        // Verify both handlers were asked
        let calls1 = handler1.get_calls();
        let calls2 = handler2.get_calls();
        assert_eq!(calls1.can_handle_calls.len(), 1);
        assert_eq!(calls2.can_handle_calls.len(), 1);

        // Only the accepting handler should have been used for transformation
        assert!(!calls1.transform_bundle_called);
        assert!(calls2.transform_bundle_called);
    }

    #[test]
    fn test_transform_slice_with_multiple_handlers_no_accepting_handler() {
        let handler = MockHandler::new_rejecting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];
        let bundle = TransformBundle::<NoEstimation>::default_all();

        let result =
            transform_slice_with_multiple_handlers([handler.clone()], &input, &mut output, &bundle);
        assert!(matches!(result, Err(TransformError::NoSupportedHandler)));

        // Verify the handler was asked but not used
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_calls.len(), 1);
        assert!(!calls.transform_bundle_called);
    }

    #[test]
    fn test_untransform_slice_with_multiple_handlers_passes_none_extension() {
        let handler = MockHandler::new_accepting("dds");
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];

        let result =
            untransform_slice_with_multiple_handlers([handler.clone()], &input, &mut output);

        // Verify the handler was called with None extension (slice operations don't have file paths)
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_untransform_calls.len(), 1);
        assert_eq!(calls.can_handle_untransform_calls[0], None);

        // Handler should reject since it expects "dds" extension but gets None
        assert!(matches!(result, Err(TransformError::NoSupportedHandler)));
    }

    #[test]
    fn test_untransform_slice_with_multiple_handlers_tries_all_handlers() {
        let handler1 = MockHandler::new_rejecting();
        let handler2 = MockHandler::new_extensionless_accepting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];

        let result = untransform_slice_with_multiple_handlers(
            [handler1.clone(), handler2.clone()],
            &input,
            &mut output,
        );
        assert!(result.is_ok());

        // Verify both handlers were asked
        let calls1 = handler1.get_calls();
        let calls2 = handler2.get_calls();
        assert_eq!(calls1.can_handle_untransform_calls.len(), 1);
        assert_eq!(calls2.can_handle_untransform_calls.len(), 1);

        // Only the accepting handler should have been used for untransformation
        assert!(!calls1.untransform_called);
        assert!(calls2.untransform_called);
    }

    #[test]
    fn test_untransform_slice_with_multiple_handlers_no_accepting_handler() {
        let handler = MockHandler::new_rejecting();
        let input = create_test_data(64);
        let mut output = vec![0u8; 64];

        let result =
            untransform_slice_with_multiple_handlers([handler.clone()], &input, &mut output);
        assert!(matches!(result, Err(TransformError::NoSupportedHandler)));

        // Verify the handler was asked but not used
        let calls = handler.get_calls();
        assert_eq!(calls.can_handle_untransform_calls.len(), 1);
        assert!(!calls.untransform_called);
    }
}

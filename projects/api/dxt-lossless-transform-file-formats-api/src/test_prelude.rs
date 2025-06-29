//! Test utilities and dummy handlers for file format API testing.

extern crate std;
use crate::bundle::TransformBundle;
use crate::embed::TransformHeader;
use crate::error::{FormatHandlerError, TransformError, TransformResult};
use crate::handlers::{FileFormatDetection, FileFormatHandler, FileFormatUntransformDetection};
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use std::sync::{Arc, Mutex};

/// Tracking information for mock handler calls.
#[derive(Debug, Clone, Default)]
pub struct MockHandlerCalls {
    pub can_handle_calls: Vec<Option<alloc::string::String>>,
    pub can_handle_untransform_calls: Vec<Option<alloc::string::String>>,
    pub transform_bundle_called: bool,
    pub untransform_called: bool,
}

/// A mock file format handler that tracks method calls and extension parameters.
///
/// This handler allows configuring acceptance behavior and tracks all method calls
/// for verification in tests.
#[derive(Debug, Clone)]
pub struct MockHandler {
    calls: Arc<Mutex<MockHandlerCalls>>,
    accepts_extension: Option<alloc::string::String>,
    accepts_transform: bool,
    accepts_untransform: bool,
}

impl MockHandler {
    /// Create a new mock handler that accepts files with the given extension.
    pub fn new_accepting(extension: &str) -> Self {
        Self {
            calls: Arc::new(Mutex::new(MockHandlerCalls::default())),
            accepts_extension: Some(extension.to_string()),
            accepts_transform: true,
            accepts_untransform: true,
        }
    }

    /// Create a new mock handler that rejects all files.
    pub fn new_rejecting() -> Self {
        Self {
            calls: Arc::new(Mutex::new(MockHandlerCalls::default())),
            accepts_extension: None,
            accepts_transform: false,
            accepts_untransform: false,
        }
    }

    /// Create a new extensionless mock handler (only checks content, no extension requirement).
    pub fn new_extensionless_accepting() -> Self {
        Self {
            calls: Arc::new(Mutex::new(MockHandlerCalls::default())),
            accepts_extension: None,
            accepts_transform: true,
            accepts_untransform: true,
        }
    }

    /// Get the recorded calls made to this handler.
    pub fn get_calls(&self) -> MockHandlerCalls {
        self.calls.lock().unwrap().clone()
    }

    /// Clear all recorded calls.
    pub fn clear_calls(&self) {
        *self.calls.lock().unwrap() = MockHandlerCalls::default();
    }
}

impl FileFormatDetection for MockHandler {
    fn can_handle(&self, _data: &[u8], file_extension: Option<&str>) -> bool {
        // Record the call with the extension that was passed
        self.calls
            .lock()
            .unwrap()
            .can_handle_calls
            .push(file_extension.map(|s| s.to_string()));

        if !self.accepts_transform {
            return false;
        }

        // If we require a specific extension, check it
        if let Some(required_ext) = &self.accepts_extension {
            file_extension == Some(required_ext.as_str())
        } else {
            // Extensionless handler - accept if no extension required
            true
        }
    }
}

impl FileFormatUntransformDetection for MockHandler {
    fn can_handle_untransform(&self, _data: &[u8], file_extension: Option<&str>) -> bool {
        // Record the call with the extension that was passed
        self.calls
            .lock()
            .unwrap()
            .can_handle_untransform_calls
            .push(file_extension.map(|s| s.to_string()));

        if !self.accepts_untransform {
            return false;
        }

        // If we require a specific extension, check it
        if let Some(required_ext) = &self.accepts_extension {
            file_extension == Some(required_ext.as_str())
        } else {
            // Extensionless handler - accept if no extension required
            true
        }
    }
}

impl FileFormatHandler for MockHandler {
    fn transform_bundle<T>(
        &self,
        input: &[u8],
        output: &mut [u8],
        _bundle: &TransformBundle<T>,
    ) -> TransformResult<()>
    where
        T: SizeEstimationOperations,
        T::Error: Debug,
    {
        self.calls.lock().unwrap().transform_bundle_called = true;

        if output.len() < input.len() {
            return Err(TransformError::FormatHandler(
                FormatHandlerError::OutputBufferTooSmall {
                    required: input.len(),
                    actual: output.len(),
                },
            ));
        }

        // Simple copy operation - don't test transformation logic
        output[..input.len()].copy_from_slice(input);
        Ok(())
    }

    fn untransform(&self, input: &[u8], output: &mut [u8]) -> TransformResult<()> {
        self.calls.lock().unwrap().untransform_called = true;

        if output.len() < input.len() {
            return Err(TransformError::FormatHandler(
                FormatHandlerError::OutputBufferTooSmall {
                    required: input.len(),
                    actual: output.len(),
                },
            ));
        }

        // Simple copy operation - don't test transformation logic
        output[..input.len()].copy_from_slice(input);
        Ok(())
    }
}

/// Create generic test data for testing.
pub fn create_test_data(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

/// Create a test BC1 transform header for testing dispatch functions.
pub fn create_test_bc1_header() -> TransformHeader {
    use crate::embed::formats::EmbeddableBc1Details;
    use dxt_lossless_transform_bc1::Bc1TransformSettings;
    let settings = Bc1TransformSettings::default();
    let details = EmbeddableBc1Details::from_settings(settings);
    details.to_header()
}

/// Create test BC1 data (must be multiple of 8 bytes).
pub fn create_test_bc1_data(num_blocks: usize) -> Vec<u8> {
    vec![0u8; num_blocks * 8]
}

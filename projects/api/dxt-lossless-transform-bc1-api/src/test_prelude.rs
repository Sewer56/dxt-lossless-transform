//! Test prelude for BC1 API tests.
//!
//! This module provides common test utilities and structures used across
//! multiple test modules to reduce code duplication.

use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};

/// A simple dummy estimator for testing purposes.
///
/// This estimator doesn't perform actual compression estimation but provides
/// a predictable implementation for testing API behavior.
pub struct DummyEstimator;

impl SizeEstimationOperations for DummyEstimator {
    type Error = &'static str;

    fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
        Ok(0) // No buffer needed for dummy estimator
    }

    unsafe fn estimate_compressed_size(
        &self,
        _input_ptr: *const u8,
        len_bytes: usize,
        _data_type: DataType,
        _output_ptr: *mut u8,
        _output_len: usize,
    ) -> Result<usize, Self::Error> {
        Ok(len_bytes) // Just return the input length
    }
}

//! BC3 automatic transform operations (safe slice-based wrapper).
//!
//! This module provides functions to automatically determine the optimal transform settings
//! for BC3 data and apply the transformation in a single operation.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc3-api::Bc3AutoTransformBuilder`.

use crate::transform::{
    transform_bc3_auto as unsafe_transform_bc3_auto, Bc3EstimateSettings, Bc3TransformSettings,
    DetermineBestTransformError,
};
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Extended error type that includes validation errors.
#[derive(Debug)]
pub enum Bc3AutoTransformError<T> {
    /// Input validation failed.
    InvalidLength(usize),
    /// Output buffer too small.
    OutputBufferTooSmall {
        /// Required buffer size.
        needed: usize,
        /// Actual buffer size provided.
        actual: usize,
    },
    /// Transform determination failed.
    DetermineBestTransform(DetermineBestTransformError<T>),
}

/// Transform BC3 data using automatically determined optimal settings.
///
/// This function tests various transform configurations and applies the one that
/// produces the smallest compressed size according to the provided estimator.
/// The transformation is applied directly to the output buffer.
///
/// # Parameters
///
/// - `input`: The BC3 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `options`: The pre-configured estimation options containing the size estimator
///   used to find the best possible transform by testing different configurations
///
/// # Returns
///
/// The [`Bc3TransformSettings`] that were used for the transformation.
///
/// # Errors
///
/// - [`DetermineBestTransformError::AllocateError`] if memory allocation fails
/// - [`DetermineBestTransformError::SizeEstimationError`] if the estimator fails
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc3::transform_bc3_auto_safe;
/// use dxt_lossless_transform_bc3::Bc3EstimateSettings;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc3::Bc3AutoTransformError;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc3AutoTransformError<LosslessTransformUtilsError>> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut output = vec![0u8; bc3_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
/// let options = Bc3EstimateSettings {
///     size_estimator: estimator,
///     use_all_decorrelation_modes: false,
/// };
///
/// let _transform_details = transform_bc3_auto_safe(&bc3_data, &mut output, &options)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc3_api::Bc3AutoTransformBuilder;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc3_api::Bc3Error;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc3Error<LosslessTransformUtilsError>> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut output = vec![0u8; bc3_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let _transform_details = Bc3AutoTransformBuilder::new(estimator)
///     .use_all_decorrelation_modes(false)
///     .transform(&bc3_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc3_auto<T>(
    input: &[u8],
    output: &mut [u8],
    options: &Bc3EstimateSettings<T>,
) -> Result<Bc3TransformSettings, Bc3AutoTransformError<T::Error>>
where
    T: SizeEstimationOperations,
{
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc3AutoTransformError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc3AutoTransformError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_transform_bc3_auto(input.as_ptr(), output.as_mut_ptr(), input.len(), options)
            .map_err(Bc3AutoTransformError::DetermineBestTransform)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

    // Mock estimator for testing
    struct MockEstimator;

    impl SizeEstimationOperations for MockEstimator {
        type Error = ();

        fn max_compressed_size(&self, input_size: usize) -> Result<usize, Self::Error> {
            Ok(input_size) // Return the input size as max compressed size
        }

        unsafe fn estimate_compressed_size(
            &self,
            _input_ptr: *const u8,
            _input_len: usize,
            _output_ptr: *mut u8,
            _output_len: usize,
        ) -> Result<usize, Self::Error> {
            Ok(100) // Return a fixed size for testing
        }
    }

    #[test]
    fn test_transform_bc3_auto_invalid_length() {
        let bc3_data = [0u8; 15]; // Invalid length (not divisible by 16)
        let mut output = [0u8; 15];
        let estimator = MockEstimator;
        let options = Bc3EstimateSettings {
            size_estimator: estimator,
            use_all_decorrelation_modes: false,
        };

        let result = transform_bc3_auto(&bc3_data, &mut output, &options);
        assert!(matches!(
            result,
            Err(Bc3AutoTransformError::InvalidLength(15))
        ));
    }

    #[test]
    fn test_transform_bc3_auto_output_too_small() {
        let bc3_data = [0u8; 16];
        let mut output = [0u8; 8]; // Too small
        let estimator = MockEstimator;
        let options = Bc3EstimateSettings {
            size_estimator: estimator,
            use_all_decorrelation_modes: false,
        };

        let result = transform_bc3_auto(&bc3_data, &mut output, &options);
        assert!(matches!(
            result,
            Err(Bc3AutoTransformError::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }
}

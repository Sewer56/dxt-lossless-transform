//! BC1 automatic transform operations.
//!
//! This module provides functions to automatically determine the optimal transform settings
//! for BC1 data and apply the transformation in a single operation.

use crate::error::Bc1Error;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{
    Bc1EstimateOptions, Bc1TransformSettings, DetermineBestTransformError,
    transform_bc1_auto as core_transform_bc1_auto,
};

/// Transform BC1 data using automatically determined optimal settings.
///
/// This function tests various transform configurations and applies the one that
/// produces the smallest compressed size according to the provided estimator.
/// The transformation is applied directly to the output buffer.
///
/// # Parameters
///
/// - `input`: The BC1 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `options`: The pre-configured estimation options
///
/// # Returns
///
/// The [`Bc1TransformSettings`] that were used for the transformation.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::OutputBufferTooSmall`] if output buffer is smaller than input
/// - [`Bc1Error::AllocationFailed`] if memory allocation fails
/// - [`Bc1Error::SizeEstimationFailed`] if the estimator fails (contains the actual estimator error)
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc1_api::{transform_bc1_auto, Bc1EstimateOptionsBuilder};
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let mut output = vec![0u8; bc1_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let options = Bc1EstimateOptionsBuilder::new()
///     .use_all_decorrelation_modes(true)
///     .build(estimator);
///
/// let transform_details = transform_bc1_auto(&bc1_data, &mut output, options)?;
/// ```
pub fn transform_bc1_auto<T>(
    input: &[u8],
    output: &mut [u8],
    options: Bc1EstimateOptions<T>,
) -> Result<Bc1TransformSettings, Bc1Error<T::Error>>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1Error::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe { core_transform_bc1_auto(input.as_ptr(), output.as_mut_ptr(), input.len(), options) }
        .map_err(|e| match e {
            DetermineBestTransformError::AllocateError(alloc_err) => {
                Bc1Error::AllocationFailed(alloc_err)
            }
            DetermineBestTransformError::SizeEstimationError(err) => {
                Bc1Error::SizeEstimationFailed(err)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::DummyEstimator;

    #[test]
    fn test_transform_bc1_auto() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let options = Bc1EstimateOptions {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = transform_bc1_auto(&bc1_data, &mut output, options);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC1 data"
        );
    }
}

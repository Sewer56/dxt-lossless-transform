//! Functions for determining optimal BC1 transform parameters.

use crate::error::Bc1Error;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{
    Bc1TransformDetails,
    determine_optimal_transform::{Bc1EstimateOptions, DetermineBestTransformError},
};

/// Determine the optimal transform parameters for BC1 data.
///
/// This function tests various transform configurations and returns the one that
/// produces the smallest compressed size according to the provided estimator.
///
/// # Parameters
///
/// - `input`: The BC1 data to analyze
/// - `estimator`: The size estimation operations to use
/// - `use_all_modes`: Whether to test all decorrelation modes (slower but potentially better)
///
/// # Returns
///
/// The optimal [`Bc1TransformDetails`] for the given data.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::AllocationFailed`] if memory allocation fails
/// - [`Bc1Error::SizeEstimationFailed`] if the estimator fails (contains the actual estimator error)
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc1_api::determine_optimal_transform;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let best_options = determine_optimal_transform(&bc1_data, estimator, false)?;
/// ```
pub fn determine_optimal_transform<T>(
    input: &[u8],
    estimator: T,
    use_all_modes: bool,
) -> Result<Bc1TransformDetails, Bc1Error<T::Error>>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    let options = Bc1EstimateOptions {
        size_estimator: estimator,
        use_all_decorrelation_modes: use_all_modes,
    };

    determine_optimal_transform_with_options(input, options)
}

/// Determine the optimal transform parameters for BC1 data using pre-configured options.
///
/// This function tests various transform configurations and returns the one that
/// produces the smallest compressed size according to the provided estimator.
///
/// # Parameters
///
/// - `input`: The BC1 data to analyze
/// - `options`: The pre-configured estimation options
///
/// # Returns
///
/// The optimal [`Bc1TransformDetails`] for the given data.
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::AllocationFailed`] if memory allocation fails
/// - [`Bc1Error::SizeEstimationFailed`] if the estimator fails (contains the actual estimator error)
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc1_api::{determine_optimal_transform_with_options, Bc1EstimateOptionsBuilder};
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let options = Bc1EstimateOptionsBuilder::new()
///     .use_all_decorrelation_modes(true)
///     .build(estimator);
///
/// let best_options = determine_optimal_transform_with_options(&bc1_data, options)?;
/// ```
pub fn determine_optimal_transform_with_options<T>(
    input: &[u8],
    options: Bc1EstimateOptions<T>,
) -> Result<Bc1TransformDetails, Bc1Error<T::Error>>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Safety: We've validated the input length
    unsafe {
        dxt_lossless_transform_bc1::determine_optimal_transform::determine_best_transform_details(
            input.as_ptr(),
            input.len(),
            core::ptr::null_mut(),
            options,
        )
    }
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
    fn test_determine_optimal_transform_with_options() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];

        let options = Bc1EstimateOptions {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = determine_optimal_transform_with_options(&bc1_data, options);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC1 data"
        );
    }
}

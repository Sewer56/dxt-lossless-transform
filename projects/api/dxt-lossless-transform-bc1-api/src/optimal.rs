//! Functions for determining optimal BC1 transform parameters.

use crate::error::Bc1Error;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{
    determine_optimal_transform::{Bc1EstimateOptions, DetermineBestTransformError},
    Bc1TransformDetails,
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
/// - [`Bc1Error::DeterminationFailed`] if the estimator fails
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc1_api::{determine_optimal_transform, MyEstimator};
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let estimator = MyEstimator::new();
///
/// let best_options = determine_optimal_transform(&bc1_data, estimator, false)?;
/// ```
pub fn determine_optimal_transform<T>(
    input: &[u8],
    estimator: T,
    use_all_modes: bool,
) -> Result<Bc1TransformDetails, Bc1Error>
where
    T: SizeEstimationOperations,
{
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    let options = Bc1EstimateOptions {
        size_estimator: estimator,
        use_all_decorrelation_modes: use_all_modes,
    };

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
        DetermineBestTransformError::SizeEstimationError(_) => {
            Bc1Error::DeterminationFailed("Size estimation failed".to_string())
        }
    })
}

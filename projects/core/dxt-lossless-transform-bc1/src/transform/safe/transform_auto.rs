//! BC1 automatic transform operations (safe slice-based wrapper).
//!
//! This module provides functions to automatically determine the optimal transform settings
//! for BC1 data and apply the transformation in a single operation.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc1-api::Bc1AutoTransformBuilder`.

use crate::transform::{
    transform_bc1_auto as unsafe_transform_bc1_auto, Bc1EstimateSettings, Bc1TransformSettings,
    DetermineBestTransformError,
};
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Extended error type that includes validation errors.
#[derive(Debug)]
pub enum Bc1AutoTransformError<T> {
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
/// - `options`: The pre-configured estimation options containing the size estimator
///   used to find the best possible transform by testing different configurations
///
/// # Returns
///
/// The [`Bc1TransformSettings`] that were used for the transformation.
///
/// # Errors
///
/// - [`DetermineBestTransformError::AllocateError`] if memory allocation fails
/// - [`DetermineBestTransformError::SizeEstimationError`] if the estimator fails
///
/// # Examples
///
/// ```
/// use dxt_lossless_transform_bc1::transform_bc1_auto_safe;
/// use dxt_lossless_transform_bc1::Bc1EstimateSettings;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc1::Bc1AutoTransformError;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc1AutoTransformError<LosslessTransformUtilsError>> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut output = vec![0u8; bc1_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
/// let options = Bc1EstimateSettings {
///     size_estimator: estimator,
///     use_all_decorrelation_modes: false,
/// };
///
/// let _transform_details = transform_bc1_auto_safe(&bc1_data, &mut output, options)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```
/// use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc1_api::Bc1Error;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc1Error<LosslessTransformUtilsError>> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut output = vec![0u8; bc1_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let _transform_details = Bc1AutoTransformBuilder::new(estimator)
///     .use_all_decorrelation_modes(false)
///     .transform(&bc1_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc1_auto<T>(
    input: &[u8],
    output: &mut [u8],
    options: Bc1EstimateSettings<T>,
) -> Result<Bc1TransformSettings, Bc1AutoTransformError<T::Error>>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1AutoTransformError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1AutoTransformError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We're passing valid slices to the unsafe function
    let result = unsafe {
        unsafe_transform_bc1_auto(input.as_ptr(), output.as_mut_ptr(), input.len(), options)
    };

    result.map_err(Bc1AutoTransformError::DetermineBestTransform)
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

        let options = Bc1EstimateSettings {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = super::transform_bc1_auto(&bc1_data, &mut output, options);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC1 data"
        );
    }
}

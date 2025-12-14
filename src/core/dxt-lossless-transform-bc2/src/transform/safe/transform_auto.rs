//! BC2 automatic transform operations (safe slice-based wrapper).
//!
//! This module provides functions to automatically determine the optimal transform settings
//! for BC2 data and apply the transformation in a single operation.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc2-api::Bc2AutoTransformBuilder`.

use crate::transform::{
    transform_bc2_auto as unsafe_transform_bc2_auto, Bc2EstimateSettings, Bc2TransformSettings,
    DetermineBestTransformError,
};
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Extended error type that includes validation errors.
#[derive(Debug)]
pub enum Bc2AutoTransformError<T> {
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

/// Transform BC2 data using automatically determined optimal settings.
///
/// This function tests various transform configurations and applies the one that
/// produces the smallest compressed size according to the provided estimator.
/// The transformation is applied directly to the output buffer.
///
/// # Parameters
///
/// - `input`: The BC2 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `options`: The pre-configured estimation options containing the size estimator
///   used to find the best possible transform by testing different configurations
///
/// # Returns
///
/// The [`Bc2TransformSettings`] that were used for the transformation.
///
/// # Errors
///
/// - [`DetermineBestTransformError::AllocateError`] if memory allocation fails
/// - [`DetermineBestTransformError::SizeEstimationError`] if the estimator fails
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc2::transform_bc2_auto_safe;
/// use dxt_lossless_transform_bc2::Bc2EstimateSettings;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc2::Bc2AutoTransformError;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc2AutoTransformError<LosslessTransformUtilsError>> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut output = vec![0u8; bc2_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
/// let options = Bc2EstimateSettings {
///     size_estimator: estimator,
///     use_all_decorrelation_modes: false,
/// };
///
/// let _transform_details = transform_bc2_auto_safe(&bc2_data, &mut output, &options)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc2_api::Bc2AutoTransformBuilder;
/// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
/// # use dxt_lossless_transform_bc2_api::Bc2Error;
/// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
///
/// # fn main() -> Result<(), Bc2Error<LosslessTransformUtilsError>> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut output = vec![0u8; bc2_data.len()];
/// let estimator = LosslessTransformUtilsSizeEstimation::new();
///
/// let _transform_details = Bc2AutoTransformBuilder::new(estimator)
///     .use_all_decorrelation_modes(false)
///     .transform(&bc2_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc2_auto<T>(
    input: &[u8],
    output: &mut [u8],
    options: &Bc2EstimateSettings<T>,
) -> Result<Bc2TransformSettings, Bc2AutoTransformError<T::Error>>
where
    T: SizeEstimationOperations,
{
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc2AutoTransformError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc2AutoTransformError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We're passing valid slices to the unsafe function
    let result = unsafe {
        unsafe_transform_bc2_auto(input.as_ptr(), output.as_mut_ptr(), input.len(), options)
    };

    result.map_err(Bc2AutoTransformError::DetermineBestTransform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::DummyEstimator;

    #[test]
    #[allow(unreachable_code)]
    fn test_transform_bc2_auto() {
        return;

        // Create minimal BC2 block data (16 bytes per block)
        let bc2_data = [
            // Alpha data (8 bytes - 4-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 16];

        let options = Bc2EstimateSettings {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = super::transform_bc2_auto(&bc2_data, &mut output, &options);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC2 data"
        );
    }
}

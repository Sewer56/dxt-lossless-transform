//! BC2 transform builder implementation.

extern crate alloc;

use crate::error::TransformError;
use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc2::Bc2TransformSettings;
use dxt_lossless_transform_bc2_api::Bc2Error;
use dxt_lossless_transform_bc2_api::{Bc2AutoTransformBuilder, Bc2ManualTransformBuilder};

/// BC2 transform builder that transparently supports both manual and automatic optimization.
///
/// This enum wraps both [`Bc2ManualTransformBuilder`] and [`Bc2AutoTransformBuilder`]
/// to provide a unified interface for BC2 transformation operations.
pub(super) enum Bc2Builder<T = NoEstimation>
where
    T: SizeEstimationOperations,
{
    /// Manual transform builder with explicit configuration
    Manual(Bc2ManualTransformBuilder),
    /// Automatic transform builder with size estimation optimization
    Auto(Bc2AutoTransformBuilder<T>),
}

impl<T> Bc2Builder<T>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    /// Transform a slice and return the transform details.
    ///
    /// This method handles both manual and automatic transform builders transparently.
    /// For automatic builders, it will find the optimal settings and apply them.
    /// For manual builders, it will use the pre-configured settings.
    ///
    /// # Parameters
    /// - `input`: Input texture data to transform
    /// - `output`: Output buffer for transformed data (must be at least the same size as input)
    ///
    /// # Returns
    /// The transform settings that were used, which can be embedded in the file header.
    pub(super) fn transform_slice_with_details(
        &self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<Bc2TransformSettings, TransformError> {
        match self {
            Bc2Builder::Manual(builder) => {
                // Get settings before transforming
                let settings = builder.get_settings();
                builder.transform(input, output)?;
                Ok(settings)
            }
            Bc2Builder::Auto(builder) => {
                let settings = builder.transform(input, output).map_err(|e| match e {
                    Bc2Error::InvalidLength(len) => {
                        TransformError::Bc2(Bc2Error::InvalidLength(len))
                    }
                    Bc2Error::OutputBufferTooSmall { needed, actual } => {
                        TransformError::Bc2(Bc2Error::OutputBufferTooSmall { needed, actual })
                    }
                    Bc2Error::AllocationFailed => TransformError::Bc2(Bc2Error::AllocationFailed),
                    Bc2Error::SizeEstimationFailed(err) => TransformError::Bc2(
                        Bc2Error::SizeEstimationFailed(alloc::format!("{err:?}")),
                    ),
                })?;
                Ok(settings.get_settings())
            }
        }
    }
}

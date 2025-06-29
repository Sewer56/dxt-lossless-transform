//! BC1 transform builder implementation.

extern crate alloc;

use crate::error::TransformError;
use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_bc1_api::Bc1Error;
use dxt_lossless_transform_bc1_api::{Bc1AutoTransformBuilder, Bc1ManualTransformBuilder};

/// BC1 transform builder that transparently supports both manual and automatic optimization.
///
/// This enum wraps both [`Bc1ManualTransformBuilder`] and [`Bc1AutoTransformBuilder`]
/// to provide a unified interface for BC1 transformation operations.
pub(super) enum Bc1Builder<T = NoEstimation>
where
    T: SizeEstimationOperations,
{
    /// Manual transform builder with explicit configuration
    Manual(Bc1ManualTransformBuilder),
    /// Automatic transform builder with size estimation optimization
    Auto(Bc1AutoTransformBuilder<T>),
}

impl<T> Bc1Builder<T>
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
    ) -> Result<Bc1TransformSettings, TransformError> {
        match self {
            Bc1Builder::Manual(builder) => {
                // Get settings before transforming
                let settings = builder.get_settings();
                builder.transform(input, output)?;
                Ok(settings)
            }
            Bc1Builder::Auto(builder) => {
                let settings = builder.transform(input, output).map_err(|e| match e {
                    Bc1Error::InvalidLength(len) => {
                        TransformError::Bc1(Bc1Error::InvalidLength(len))
                    }
                    dxt_lossless_transform_bc1_api::Bc1Error::OutputBufferTooSmall {
                        needed,
                        actual,
                    } => TransformError::Bc1(Bc1Error::OutputBufferTooSmall { needed, actual }),
                    Bc1Error::AllocationFailed => TransformError::Bc1(Bc1Error::AllocationFailed),
                    Bc1Error::SizeEstimationFailed(err) => TransformError::Bc1(
                        Bc1Error::SizeEstimationFailed(alloc::format!("{err:?}")),
                    ),
                })?; // This mapping is a bit nasty but forced by the generic on Bc1Error deep down.
                Ok(settings.get_settings())
            }
        }
    }
}

//! Transform bundle for handling multiple BCx formats.

extern crate alloc;

use core::marker::PhantomData;
use dxt_lossless_transform_api_common::estimate::{NoEstimation, SizeEstimationOperations};
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;
use dxt_lossless_transform_bc2_api::Bc2ManualTransformBuilder;

use crate::embed::{TransformFormat, TransformHeader};
use crate::error::{FormatHandlerError, TransformError, TransformResult};

// Re-export BC1 and BC2 builders (used externally)
use bc1::Bc1Builder;
use bc2::Bc2Builder;

// Import BC3-7 builders for internal use in PhantomData fields
use bc3::Bc3TransformBuilder;
use bc7::Bc7TransformBuilder;

// Submodules for each BCx format
mod bc1; // BC1 module stays lic (Bc1Builder is used externally)
mod bc2; // BC2-7 modules are private (builders not used externally)
mod bc3;
mod bc7;

/// Bundle of transform builders for different BCx formats.
///
/// This allows configuring transform settings for supported BCx formats,
/// and the file format handler will use the appropriate one based on the
/// detected format.
///
/// The type parameter `T` specifies the size estimation strategy:
///
/// - For manual-only use cases: `TransformBundle<NoEstimation>`
/// - For auto-optimization with size estimation: `TransformBundle<MyEstimator>`
pub struct TransformBundle<T>
where
    T: SizeEstimationOperations,
{
    /// BC1 transform builder (supports both manual and automatic modes)
    bc1: Option<Bc1Builder<T>>,
    /// BC2 transform builder (supports both manual and automatic modes)
    bc2: Option<Bc2Builder<T>>,
    /// BC3 transform builder (placeholder for future implementation)  
    bc3: PhantomData<Bc3TransformBuilder>,
    /// BC7 transform builder (placeholder for future implementation)
    bc7: PhantomData<Bc7TransformBuilder>,
}

impl<T> Default for TransformBundle<T>
where
    T: SizeEstimationOperations,
{
    fn default() -> Self {
        Self {
            bc1: None,
            bc2: None,
            bc3: PhantomData,
            bc7: PhantomData,
        }
    }
}

impl<T> TransformBundle<T>
where
    T: SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    /// Create a new empty bundle.
    ///
    /// You can then set individual builders as needed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set BC1 manual transform builder
    pub fn with_bc1_manual(
        mut self,
        builder: dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder,
    ) -> Self {
        self.bc1 = Some(Bc1Builder::Manual(builder));
        self
    }

    /// Set BC1 automatic transform builder
    pub fn with_bc1_auto(
        mut self,
        builder: dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder<T>,
    ) -> Self {
        self.bc1 = Some(Bc1Builder::Auto(builder));
        self
    }

    /// Set BC2 manual transform builder
    pub fn with_bc2_manual(
        mut self,
        builder: dxt_lossless_transform_bc2_api::Bc2ManualTransformBuilder,
    ) -> Self {
        self.bc2 = Some(Bc2Builder::Manual(builder));
        self
    }

    /// Set BC2 automatic transform builder
    pub fn with_bc2_auto(
        mut self,
        builder: dxt_lossless_transform_bc2_api::Bc2AutoTransformBuilder<T>,
    ) -> Self {
        self.bc2 = Some(Bc2Builder::Auto(builder));
        self
    }

    /// Dispatch transform operation based on the detected format.
    ///
    /// This method handles the transform operation and returns the transform header
    /// that should be embedded in the file.
    ///
    /// # Parameters
    /// - `format`: The detected texture format to transform
    /// - `input_texture_data`: Input texture data to transform
    /// - `output_texture_data`: Output buffer for transformed data (must be at least the same size as input)
    ///
    /// # Returns
    /// A [`TransformHeader`] containing the transform details for embedding.
    pub(super) fn dispatch_transform(
        &self,
        format: TransformFormat,
        input_texture_data: &[u8],
        output_texture_data: &mut [u8],
    ) -> TransformResult<TransformHeader> {
        if output_texture_data.len() < input_texture_data.len() {
            return Err(TransformError::FormatHandler(
                FormatHandlerError::OutputBufferTooSmall {
                    required: input_texture_data.len(),
                    actual: output_texture_data.len(),
                },
            ));
        }

        let header = match format {
            TransformFormat::Bc1 => {
                let builder = self
                    .bc1
                    .as_ref()
                    .ok_or(FormatHandlerError::NoBuilderForFormat(TransformFormat::Bc1))?;

                let details = builder
                    .transform_slice_with_details(input_texture_data, output_texture_data)?;

                crate::embed::EmbeddableBc1Details::from_settings(details).to_header()
            }
            TransformFormat::Bc2 => {
                let builder = self
                    .bc2
                    .as_ref()
                    .ok_or(FormatHandlerError::NoBuilderForFormat(TransformFormat::Bc2))?;

                let details = builder
                    .transform_slice_with_details(input_texture_data, output_texture_data)?;

                crate::embed::formats::EmbeddableBc2Details::from_settings(details).to_header()
            }
            _ => {
                return Err(TransformError::UnknownTransformFormat);
            }
        };

        Ok(header)
    }
}

impl TransformBundle<NoEstimation> {
    /// Create a bundle with sensible manual settings for supported formats.
    ///
    /// This is mainly intended for testing and scenarios where only manual
    /// configuration is needed. Only manual transform operations are supported
    /// with this mode - automatic optimization features will not function.
    ///
    /// Currently BC1 and BC2 are supported with default manual configuration.
    pub fn default_all() -> Self {
        Self {
            bc1: Some(Bc1Builder::Manual(Bc1ManualTransformBuilder::new())),
            bc2: Some(Bc2Builder::Manual(Bc2ManualTransformBuilder::new())),
            bc3: PhantomData,
            bc7: PhantomData,
        }
    }
}

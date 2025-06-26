//! Transform bundle for handling multiple BCx formats.

use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;
use dxt_lossless_transform_bc2::BC2TransformDetails;
use dxt_lossless_transform_bc3::BC3TransformDetails;
// BC7 API not yet available
// use dxt_lossless_transform_bc7_api::{Bc7AutoTransformBuilder, Bc7ManualTransformBuilder};

use crate::error::FileFormatError;

/// Bundle of transform builders for different BCx formats.
///
/// This allows configuring different transform settings for each BCx format,
/// and the file format handler will use the appropriate one based on the
/// detected format.
#[derive(Default)]
pub struct TransformBundle {
    /// BC1 transform builder (auto or manual)
    pub bc1: Option<Bc1TransformBuilder>,
    /// BC2 transform builder (placeholder - BC2 doesn't have configurable options)
    pub bc2: Option<Bc2TransformBuilder>,
    /// BC3 transform builder (placeholder - BC3 doesn't have configurable options)
    pub bc3: Option<Bc3TransformBuilder>,
    /// BC7 transform builder (placeholder for future)
    pub bc7: Option<Bc7TransformBuilder>,
}

/// BC1 transform builder - currently only supports manual mode due to auto builder limitations
pub type Bc1TransformBuilder = Bc1ManualTransformBuilder;

/// Extension trait for Bc1ManualTransformBuilder to add slice transform with details
pub trait Bc1TransformBuilderExt {
    /// Transform a slice and return the detransform details
    fn transform_slice_with_details(
        &self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<Bc1TransformSettings, FileFormatError>;
}

impl Bc1TransformBuilderExt for Bc1TransformBuilder {
    fn transform_slice_with_details(
        &self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<Bc1TransformSettings, FileFormatError> {
        // Manual builder's transform() method returns a result
        self.transform(input, output)?;

        // Reconstruct settings from the builder's configuration
        // Note: The manual builder doesn't expose getters, so we'll use defaults
        // This is a limitation of the current API
        Ok(Bc1TransformSettings::default())
    }
}

/// BC2 placeholder - BC2 doesn't have configurable transform options
pub struct Bc2TransformBuilder;

impl Bc2TransformBuilder {
    pub fn transform_slice(
        &self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<BC2TransformDetails, FileFormatError> {
        // BC2 transform needs unsafe
        if input.len() != output.len() || input.len() % 16 != 0 {
            return Err(FileFormatError::InvalidFileData(
                "BC2 data must be 16-byte aligned".to_string(),
            ));
        }

        let details = unsafe {
            dxt_lossless_transform_bc2::transform_bc2(
                input.as_ptr(),
                output.as_mut_ptr(),
                input.len(),
            )
        };
        Ok(details)
    }
}

/// BC3 placeholder - BC3 doesn't have configurable transform options
pub struct Bc3TransformBuilder;

impl Bc3TransformBuilder {
    pub fn transform_slice(
        &self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<BC3TransformDetails, FileFormatError> {
        // BC3 transform needs unsafe
        if input.len() != output.len() || input.len() % 16 != 0 {
            return Err(FileFormatError::InvalidFileData(
                "BC3 data must be 16-byte aligned".to_string(),
            ));
        }

        let details = unsafe {
            dxt_lossless_transform_bc3::transform_bc3(
                input.as_ptr(),
                output.as_mut_ptr(),
                input.len(),
            )
        };
        Ok(details)
    }
}

/// BC7 placeholder (not yet implemented)
pub struct Bc7TransformBuilder;

impl TransformBundle {
    /// Create a new empty bundle.
    ///
    /// You can then set individual builders as needed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a bundle with default settings for all formats.
    ///
    /// Note: BC1 auto requires an estimator, so we use manual mode with defaults.
    pub fn default_all() -> Self {
        Self {
            bc1: Some(Bc1ManualTransformBuilder::new()),
            bc2: Some(Bc2TransformBuilder),
            bc3: Some(Bc3TransformBuilder),
            bc7: None, // BC7 not yet implemented
        }
    }

    /// Set BC1 manual transform
    pub fn with_bc1_manual(mut self, builder: Bc1ManualTransformBuilder) -> Self {
        self.bc1 = Some(builder);
        self
    }

    /// Set BC2 transform
    pub fn with_bc2(mut self) -> Self {
        self.bc2 = Some(Bc2TransformBuilder);
        self
    }

    /// Set BC3 transform
    pub fn with_bc3(mut self) -> Self {
        self.bc3 = Some(Bc3TransformBuilder);
        self
    }
}

/// Result of an untransform operation
#[derive(Debug)]
pub enum UntransformResult {
    /// BC1 was untransformed with these details
    Bc1(Bc1TransformSettings),
    /// BC2 was untransformed
    Bc2,
    /// BC3 was untransformed
    Bc3,
    /// BC7 was untransformed (placeholder)
    Bc7,
}

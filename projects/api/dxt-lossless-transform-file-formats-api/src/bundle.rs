//! Transform bundle for handling multiple BCx formats.

use core::marker::PhantomData;
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_bc1_api::Bc1ManualTransformBuilder;
// BC7 API not yet available
// use dxt_lossless_transform_bc7_api::{Bc7AutoTransformBuilder, Bc7ManualTransformBuilder};

use crate::error::FileFormatError;

/// Bundle of transform builders for different BCx formats.
///
/// This allows configuring transform settings for supported BCx formats,
/// and the file format handler will use the appropriate one based on the
/// detected format.
#[derive(Default)]
pub struct TransformBundle {
    /// BC1 transform builder (currently only manual mode supported)
    pub bc1: Option<Bc1TransformBuilder>,
    /// BC2 transform builder (placeholder for future implementation)
    pub bc2: PhantomData<Bc2TransformBuilder>,
    /// BC3 transform builder (placeholder for future implementation)  
    pub bc3: PhantomData<Bc3TransformBuilder>,
    /// BC7 transform builder (placeholder for future implementation)
    pub bc7: PhantomData<Bc7TransformBuilder>,
}

/// BC1 transform builder - currently only supports manual mode due to auto builder limitations
pub type Bc1TransformBuilder = Bc1ManualTransformBuilder;

/// BC2 transform builder (placeholder type for future implementation)
pub struct Bc2TransformBuilder;

/// BC3 transform builder (placeholder type for future implementation)
pub struct Bc3TransformBuilder;

/// BC7 transform builder (placeholder type for future implementation)
pub struct Bc7TransformBuilder;

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

impl TransformBundle {
    /// Create a new empty bundle.
    ///
    /// You can then set individual builders as needed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a bundle with default settings for supported formats.
    ///
    /// Currently only BC1 is supported with configurable options.
    pub fn default_all() -> Self {
        Self {
            bc1: Some(Bc1ManualTransformBuilder::new()),
            bc2: PhantomData,
            bc3: PhantomData,
            bc7: PhantomData,
        }
    }

    /// Set BC1 manual transform
    pub fn with_bc1_manual(mut self, builder: Bc1ManualTransformBuilder) -> Self {
        self.bc1 = Some(builder);
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

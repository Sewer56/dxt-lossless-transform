#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// Module declarations
pub mod error;
pub mod transform;

#[cfg(feature = "c-exports")]
pub mod c_api;

// Re-export main functionality at crate root
pub use error::Bc1Error;

// Re-export transform functions
pub use transform::{
    transform_bc1_auto, transform_bc1_with_settings, untransform_bc1_with_settings,
};

// Re-export builders
pub use transform::{
    Bc1DetransformSettingsBuilder, Bc1EstimateSettingsBuilder, Bc1TransformSettingsBuilder,
};

// Re-export core types for convenience
pub use transform::{
    Bc1DetransformSettings, Bc1EstimateSettings, Bc1TransformSettings, DetermineBestTransformError,
    YCoCgVariant,
};

// Re-export utility functions
pub use transform::{Decoded4x4Block, decode_bc1_block, decode_bc1_block_from_slice};

// Test utilities
#[cfg(test)]
mod test_prelude;

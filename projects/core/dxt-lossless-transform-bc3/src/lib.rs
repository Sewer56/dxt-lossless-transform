#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![no_std]
// Not yet in stable today, but will be in 1.89.0
#![allow(stable_features)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")),
    feature(stdarch_x86_avx512)
)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "experimental")]
pub mod experimental;

/// Provides optimized routines to transform/untransform into various forms of the lossless transform.
pub mod transform;

pub mod util;

// Re-export the new transform infrastructure
pub use transform::settings::{Bc3TransformSettings, Bc3UntransformSettings};
pub use transform::transform_auto::{
    transform_bc3_auto, Bc3EstimateSettings, DetermineBestTransformError,
};
pub use transform::transform_with_settings::{
    transform_bc3_with_settings, untransform_bc3_with_settings,
};

// Re-export safe module functions
pub use transform::{
    transform_bc3_auto_safe, transform_bc3_with_settings_safe, untransform_bc3_with_settings_safe,
    Bc3AutoTransformError, Bc3ValidationError,
};

// Re-export functions for benchmarking when the 'bench' feature is enabled
#[cfg(feature = "bench")]
pub mod bench;

#[cfg(test)]
pub mod test_prelude;

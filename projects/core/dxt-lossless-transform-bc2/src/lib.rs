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

/// Advanced BC2 transform operations with settings, auto-optimization, and safe wrappers
pub mod transform;

#[cfg(feature = "bench")]
pub mod bench;

/// C API functions for BC2 transforms (enabled with c-exports feature)
#[cfg(feature = "c-exports")]
pub mod c_api;

pub mod util;

#[cfg(test)]
pub mod test_prelude;

// Re-export transform module contents for advanced BC2 operations
pub use transform::{
    transform_bc2_auto, transform_bc2_auto_safe, transform_bc2_with_settings,
    transform_bc2_with_settings_safe, untransform_bc2_with_settings,
    untransform_bc2_with_settings_safe, Bc2AutoTransformError, Bc2EstimateSettings,
    Bc2TransformSettings, Bc2UntransformSettings, Bc2ValidationError, DetermineBestTransformError,
};

#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
// Not yet in stable today, but will be in 1.89.0
#![allow(stable_features)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")),
    feature(stdarch_x86_avx512)
)]
#![warn(missing_docs)]

pub(crate) mod transform;

#[cfg(feature = "bench")]
pub mod bench;
#[cfg(feature = "experimental")]
pub mod experimental;
pub mod util;

// Re-export main types and functions from transform module
pub use transform::*;

// Re-export YCoCgVariant for convenience
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Common test prelude for avoiding duplicate imports in test modules
#[cfg(test)]
pub(crate) mod test_prelude;

#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![no_std]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

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

/// C API exports
#[cfg(feature = "c-exports")]
pub mod c_api;

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

// Re-export BUILDERS (stable, recommended)
pub use transform::{Bc1AutoTransformBuilder, Bc1ManualTransformBuilder};

// Re-export only essential types (YCoCgVariant for builder configuration)
pub use transform::YCoCgVariant;

#![doc = include_str!("../README.MD")]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

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

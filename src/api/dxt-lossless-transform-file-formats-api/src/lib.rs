#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

// Core modules
pub mod api;
pub mod bundle;
pub mod embed;
pub mod error;
pub mod handlers;

#[cfg(feature = "file-io")]
pub mod file_io;

// Re-export key APIs
pub use api::*; // convenience functions
pub use bundle::TransformBundle;
pub use error::*; // error types
pub use handlers::*; // file format handler infrastructure (traits + dispatch functions)

// Test utilities (only available during testing)
#[cfg(test)]
pub mod test_prelude;

#![doc = include_str!("../README.MD")]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

pub mod allocate;
pub mod estimate;

/// C API for the lossless transform API.
#[cfg(feature = "c-exports")]
pub mod c_api;

pub mod reexports;

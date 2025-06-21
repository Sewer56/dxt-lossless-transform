#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

pub mod allocate;
pub mod estimate;

/// C API for the lossless transform API.
#[cfg(feature = "c-exports")]
pub mod c_api;

pub mod reexports;

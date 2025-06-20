#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

pub mod allocate;
pub mod estimate;

#[cfg(feature = "c-exports")]
pub mod c_api;

pub mod reexports;

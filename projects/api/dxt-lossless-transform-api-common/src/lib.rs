#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]

/// Size estimation operations for file compression.
pub mod estimate;

/// C exports for DDS functionality.
#[cfg(feature = "c-exports")]
pub mod exports;

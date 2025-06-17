#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]

/// C exports for DDS functionality.
#[cfg(feature = "c-exports")]
pub mod exports;

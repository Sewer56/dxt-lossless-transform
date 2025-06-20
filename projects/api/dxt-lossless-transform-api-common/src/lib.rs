#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(allocator_api))]

/// Memory allocation utilities with cache line alignment support.
pub mod allocate;

/// Size estimation operations for file compression.
pub mod estimate;

/// C exports for DDS functionality.
#[cfg(feature = "c-exports")]
pub mod c_api;

/// Stable re-exports of types from internal crates.
///
/// This module contains stable versions of types that are used in public APIs
/// but defined in internal crates that may change. See the module documentation
/// for details on the stability guarantees.
pub mod reexports;

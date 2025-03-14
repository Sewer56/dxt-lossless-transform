#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "c-exports")]
pub mod exports;

pub mod api;
pub use api::*;

#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![no_std]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
#![warn(missing_docs)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

pub mod color_565;
pub mod color_8888;
pub mod decoded_4x4_block;

/// This module contains various 'transforms' which may be helpful for making data more compressible.
pub mod transforms {
    pub mod split_565_color_endpoints;
}
pub mod allocate;
pub mod cpu_detect;
pub mod intrinsics;

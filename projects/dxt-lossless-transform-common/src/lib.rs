#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
// Not yet in stable today, but will be in 1.89.0
#![allow(stable_features)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")),
    feature(stdarch_x86_avx512)
)]
#![cfg_attr(feature = "nightly", feature(allocator_api))]

pub mod color_565;
pub mod color_8888;
pub mod decoded_4x4_block;
pub mod transforms {
    pub mod split_565_color_endpoints;
}
pub mod allocate;
pub mod cpu_detect;
pub mod intrinsics;

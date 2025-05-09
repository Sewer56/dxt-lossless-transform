#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(avx512_target_feature))]
#![cfg_attr(feature = "nightly", feature(stdarch_x86_avx512))]

pub mod color_565;
pub mod color_8888;
pub mod decoded_4x4_block;
pub mod transforms {
    pub mod split_565_color_endpoints;
}

#[cfg(test)]
mod tests;

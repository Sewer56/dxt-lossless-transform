#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]
#![allow(missing_docs)]

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

pub mod portable32;
pub use portable32::*;

// Wrapper functions for benchmarks
#[cfg(target_arch = "x86_64")]
pub unsafe fn sse2_shuffle_v3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::shuffle_v3(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn sse2_shuffle_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::shuffle_v2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx2_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx2::shuffle(input_ptr, output_ptr, len)
}

pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn permute_512_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::permute_512_v2(input_ptr, output_ptr, len)
}

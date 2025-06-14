#![allow(clippy::missing_safety_doc)]

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx2::*;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

pub mod portable32;
pub use portable32::*;
pub mod portable64;
pub use portable64::*;

// Wrapper functions for benchmarks
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn shufps_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::shufps_unroll_4(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn shuffle_permute_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx2::shuffle_permute_unroll_2(input_ptr, output_ptr, len)
}

pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::permute_512(input_ptr, output_ptr, len)
}

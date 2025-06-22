#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) mod avx512;

pub(crate) mod portable32;
pub(crate) mod portable64;

// Wrapper functions for benchmarks
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn shufps_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::shufps_unroll_4(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn shuffle_permute_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    super::avx2::shuffle_permute_unroll_2(input_ptr, output_ptr, len)
}

pub(crate) unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::permute_512(input_ptr, output_ptr, len)
}

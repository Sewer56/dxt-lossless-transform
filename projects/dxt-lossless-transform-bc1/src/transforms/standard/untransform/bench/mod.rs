#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]

pub mod portable32;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

// Wrapper functions for benchmarks
pub(crate) unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32_detransform(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn unpck_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    super::sse2::unpck_detransform_unroll_2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn permd_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    super::avx2::permd_detransform_unroll_2(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) unsafe fn permute_512_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    super::avx512::permute_512_detransform_unroll_2(input_ptr, output_ptr, len)
}

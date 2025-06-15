#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]

pub mod portable;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    portable::u32_detransform(input_ptr, output_ptr, len)
}

pub unsafe fn u32_detransform_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable::u32_detransform_v2(input_ptr, output_ptr, len)
}

pub unsafe fn u64_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    portable::u64_detransform(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn u32_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    sse2::u32_detransform_sse2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn u64_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::u64_detransform_sse2(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::avx512_detransform(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_detransform_32_vbmi(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::avx512_detransform_32_vbmi(input_ptr, output_ptr, len)
}

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_detransform_32_vl(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::avx512_detransform_32_vl(input_ptr, output_ptr, len)
}

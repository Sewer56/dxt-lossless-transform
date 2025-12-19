#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]
#![allow(missing_docs)]

pub mod portable;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

pub unsafe fn u32_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    portable::u32_untransform(input_ptr, output_ptr, len)
}

pub unsafe fn u32_untransform_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32_untransform_v2(input_ptr, output_ptr, len)
}

pub unsafe fn u64_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    portable::u64_untransform(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn u32_untransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    sse2::u32_untransform_sse2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn u64_untransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::sse2::u64_untransform_sse2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512vbmi::avx512_untransform(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_untransform_32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512vbmi::avx512_untransform_32(input_ptr, output_ptr, len)
}

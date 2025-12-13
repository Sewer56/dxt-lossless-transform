#![allow(clippy::missing_safety_doc)]
#![cfg(not(tarpaulin_include))]
#![allow(missing_docs)]

pub mod portable32;

pub unsafe fn u32_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::portable32::u32(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn u32_avx2_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx2::u32_avx2(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub unsafe fn avx512_vbmi_transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    super::avx512::avx512_vbmi(input_ptr, output_ptr, len)
}

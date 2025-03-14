use core::ptr::copy_nonoverlapping;
pub use dxt_lossless_transform_dds::dds::*;

/// Transforms data from the standard format to one that is more suitable for compression.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
/// - `format` must be a valid [`DdsFormat`]
#[inline]
pub unsafe fn transform_format(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    format: DdsFormat,
) {
    match format {
        DdsFormat::Unknown => { /* no-op */ }
        DdsFormat::BC1 => {
            dxt_lossless_transform_bc1::split_blocks::split_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC2 => {
            dxt_lossless_transform_bc2::split_blocks::split_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC3 => {
            dxt_lossless_transform_bc3::split_blocks::split_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC7 => {
            copy_nonoverlapping(input_ptr, output_ptr, len);
        }
    }
}

/// Untransforms data from a compression suitable one to the standard format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
/// - `format` must be a valid [`DdsFormat`], and the same format passed to [`transform_format`].
#[inline]
pub unsafe fn untransform_format(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    format: DdsFormat,
) {
    match format {
        DdsFormat::Unknown => { /* no-op */ }
        DdsFormat::BC1 => {
            dxt_lossless_transform_bc1::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC2 => {
            dxt_lossless_transform_bc2::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC3 => {
            dxt_lossless_transform_bc3::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
        }
        DdsFormat::BC7 => todo!(),
    }
}

/// Transform BC1 data from standard interleaved format to separated color/index format
/// to improve compression ratio.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc1::split_blocks::split_blocks(input_ptr, output_ptr, len)
}

/// Transform BC1 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 8 (BC1 block size)
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
#[inline]
pub unsafe fn untransform_bc1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc1::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
}

/// Transform BC2 data from standard interleaved format to separated color/index format
/// to improve compression ratio.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc2::split_blocks::split_blocks(input_ptr, output_ptr, len)
}

/// Transform BC2 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
#[inline]
pub unsafe fn untransform_bc2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc2::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
}

/// Transform BC3 data from standard interleaved format to separated color/index format
/// to improve compression ratio.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC3 block size)
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc3::split_blocks::split_blocks(input_ptr, output_ptr, len)
}

/// Transform BC3 data from separated color/index format back to standard interleaved format.
///
/// This function selects the best available implementation based on available CPU features.
/// Hardware accelerated (SIMD) methods are currently available for x86 and x86-64.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of len bytes
/// - `output_ptr` must be valid for writes of len bytes
/// - `len` must be divisible by 16 (BC3 block size)
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
#[inline]
pub unsafe fn untransform_bc3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform_bc3::split_blocks::unsplit_blocks(input_ptr, output_ptr, len)
}

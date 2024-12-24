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
/// - `input_ptr` and `output_ptr` must be 64-byte aligned (for performance and required by some platforms).
#[inline]
pub unsafe fn transform_bc1(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    dxt_lossless_transform::raw::bc1::transform_bc1(input_ptr, output_ptr, len)
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
    dxt_lossless_transform::raw::bc1::untransform_bc1(input_ptr, output_ptr, len)
}

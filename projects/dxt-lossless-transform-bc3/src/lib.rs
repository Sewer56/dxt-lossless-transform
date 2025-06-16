#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
// Not yet in stable today, but will be in 1.89.0
#![allow(stable_features)]
#![cfg_attr(
    all(feature = "nightly", any(target_arch = "x86_64", target_arch = "x86")),
    feature(stdarch_x86_avx512)
)]

#[cfg(feature = "experimental")]
pub mod experimental;
pub mod transforms;
pub mod util;

/// The information about the BC3 transform that was just performed.
/// Each item transformed via [`transform_bc3`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass the same instance to [`untransform_bc3`].
pub struct BC3TransformDetails {}

/// Transform BC3 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `output_ptr`: A pointer to the output data (output BC3 blocks)
/// - `len`: The length of the input data in bytes
///
/// # Returns
///
/// A struct informing you how the file was transformed. You will need this to call the
/// [`untransform_bc3`] function.
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc3(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) -> BC3TransformDetails {
    debug_assert!(len % 16 == 0);
    crate::transforms::standard::split_blocks(input_ptr, output_ptr, len);
    BC3TransformDetails {}
}

/// Untransform BC3 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks).
///   Output from [`transform_bc3`].
/// - `output_ptr`: A pointer to the output data (output BC3 blocks)
/// - `len`: The length of the input data in bytes
/// - `_details`: A struct containing information about the transform that was performed
///   obtained from the original call to [`transform_bc3`].
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc3(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _details: BC3TransformDetails,
) {
    debug_assert!(len % 16 == 0);
    crate::transforms::standard::unsplit_blocks(input_ptr, output_ptr, len);
}

#[cfg(test)]
pub mod test_prelude;

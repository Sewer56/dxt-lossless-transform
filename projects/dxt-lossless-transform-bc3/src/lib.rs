#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(avx512_target_feature))]
#![cfg_attr(feature = "nightly", feature(stdarch_x86_avx512))]

pub mod normalize_blocks;
pub mod split_blocks;
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
    split_blocks::split_blocks(input_ptr, output_ptr, len);
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
    split_blocks::unsplit_blocks(input_ptr, output_ptr, len);
}

#[cfg(test)]
mod testutils {
    use core::alloc::Layout;
    use safe_allocator_api::RawAlloc;

    pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
        let layout = Layout::from_size_align(num_bytes, 64).unwrap();
        RawAlloc::new(layout).unwrap()
    }
}

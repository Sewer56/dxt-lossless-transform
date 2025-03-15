#![doc = include_str!(concat!("../", std::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]

use core::alloc::Layout;

use safe_allocator_api::RawAlloc;
use split_blocks::{split::split_blocks, unsplit_blocks};
pub mod normalize_blocks;
pub mod split_blocks;
pub mod util;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`transform_bc1`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass the same instance to [`untransform_bc1`].
pub struct Bc1TransformDetails {}

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
///
/// # Returns
///
/// A struct informing you how the file was transformed. You will need this to call the
/// [`untransform_bc1`] function.
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
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) -> Bc1TransformDetails {
    debug_assert!(len % 8 == 0);

    let mut normalized = RawAlloc::new(Layout::from_size_align_unchecked(len, 64)).unwrap();
    normalize_blocks::normalize_blocks(input_ptr, normalized.as_mut_ptr(), len, false);
    split_blocks(normalized.as_ptr(), output_ptr, len);
    Bc1TransformDetails {}
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `_details`: A struct containing information about the transform that was performed
///   obtained from the original call to [`transform_bc1`].
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
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _details: &Bc1TransformDetails,
) {
    debug_assert!(len % 8 == 0);
    unsplit_blocks(input_ptr, output_ptr, len);
}

#[cfg(test)]
mod testutils {
    use safe_allocator_api::RawAlloc;
    use std::alloc::Layout;

    pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
        let layout = Layout::from_size_align(num_bytes, 64).unwrap();
        RawAlloc::new(layout).unwrap()
    }
}

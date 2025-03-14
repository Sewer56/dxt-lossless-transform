#![doc = include_str!("../../../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]

pub mod split_blocks;

/// The information about the BC3 transform that was just performed.
/// Each item transformed via [`transform_bc3`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass the same instance to [`untransform_bc3`].
pub struct BC3TransformDetails {}

/// Transform bc3 data from standard interleaved format to separated alpha/color/index format
/// using the best known implementation for the current CPU.
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

/// Transform bc3 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
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
    use safe_allocator_api::RawAlloc;
    use std::alloc::Layout;

    pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
        let layout = Layout::from_size_align(num_bytes, 64).unwrap();
        RawAlloc::new(layout).unwrap()
    }
}

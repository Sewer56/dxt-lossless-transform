//! Helper methods around memory allocation.
//!
//! Provides features like:
//! - Aligned allocations
//! - Allocations of arrays of pointers
//!
//! etc.

use core::alloc::{Layout, LayoutError};
use core::mem::MaybeUninit;
use safe_allocator_api::allocator_api::*;
use safe_allocator_api::RawAlloc;
use thiserror::Error;

/// Represents a fixed-size array of [`RawAlloc`]s.
pub struct FixedRawAllocArray<const NUM_ELEMENTS: usize> {
    /// The underlying raw allocations.
    pub allocations: [RawAlloc; NUM_ELEMENTS],
}

impl<const NUM_ELEMENTS: usize> FixedRawAllocArray<NUM_ELEMENTS> {
    /// Allocates a [`FixedRawAllocArray`] holding an inner array of [`RawAlloc`]s.
    ///
    /// # Parameters
    ///
    /// - `num_bytes`: The number of bytes to allocate in each element
    ///
    /// # Returns
    ///
    /// A [`FixedRawAllocArray`] containing the allocated data
    #[inline]
    pub fn new(num_bytes: usize) -> Result<Self, AllocateError> {
        #[allow(clippy::uninit_assumed_init)]
        // RawAlloc has no default, and we initialize all elements below
        let mut allocations =
            unsafe { MaybeUninit::<[RawAlloc; NUM_ELEMENTS]>::uninit().assume_init() };
        for item in allocations.iter_mut().take(NUM_ELEMENTS) {
            *item = allocate_align_64(num_bytes)?;
        }
        Ok(Self { allocations })
    }

    /// Gets a slice of just the pointers to the start of each allocation.
    ///
    /// # Returns
    ///
    /// A slice of [`RawAlloc`]s
    #[inline]
    pub fn get_pointer_slice(&mut self) -> [*mut u8; NUM_ELEMENTS] {
        core::array::from_fn(|x| self.allocations[x].as_mut_ptr())
    }
}

/// Allocates data with an alignment of 64 bytes.
///
/// # Parameters
///
/// - `num_bytes`: The number of bytes to allocate
///
/// # Returns
///
/// A [`RawAlloc`] containing the allocated data
pub fn allocate_align_64(num_bytes: usize) -> Result<RawAlloc, AllocateError> {
    let layout = Layout::from_size_align(num_bytes, 64)?;
    Ok(RawAlloc::new(layout)?)
}

/// An error that happened in memory allocation within the library.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AllocateError {
    #[error("Invalid layout provided. Likely due to `num_bytes` in `allocate_align_64` being larger than isize::MAX. {0}")]
    LayoutError(#[from] LayoutError),

    #[error(transparent)]
    AllocationFailed(#[from] AllocError),
}

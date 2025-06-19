//! Memory allocation utilities for DXT lossless transform operations.
//!
//! This crate provides memory allocation features that Rust does not have out of the box,
//! which are useful for my purposes.
//!
//! ## Useful APIs
//!
//! [`allocate_align_64`]: Allocates uninitialized memory aligned to 64-bytes.
//! [`FixedRawAllocArray::new`]: Creates a new array of aligned allocations.
//!
//! ## Safety
//!
//! All allocation operations are wrapped in safe APIs that handle proper initialization,
//! cleanup, and error handling. Memory is automatically deallocated when the allocation
//! wrappers are dropped.

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
        // Use MaybeUninit array to avoid double-drop issues
        let mut allocations: [MaybeUninit<RawAlloc>; NUM_ELEMENTS] =
            core::array::from_fn(|_| MaybeUninit::uninit());

        // Track how many allocations we've successfully made for cleanup on failure
        let mut initialized_count = 0;

        for item in allocations.iter_mut() {
            match allocate_align_64(num_bytes) {
                Ok(alloc) => {
                    item.write(alloc);
                    initialized_count += 1;
                }
                Err(e) => {
                    // Clean up any previously allocated memory by dropping initialized elements
                    for cleanup_item in &mut allocations[0..initialized_count] {
                        unsafe {
                            cleanup_item.assume_init_drop();
                        }
                    }
                    return Err(e);
                }
            }
        }

        // All elements are now initialized, safely convert to final array
        let allocations = allocations.map(|item| unsafe { item.assume_init() });
        Ok(Self { allocations })
    }

    /// Gets a slice of just the pointers to the start of each allocation.
    ///
    /// # Safety
    ///
    /// The returned pointers are valid only as long as this [`FixedRawAllocArray`]
    /// instance remains alive. Using these pointers after the instance is dropped
    /// results in undefined behavior.
    ///
    /// # Returns
    ///
    /// An array of raw pointers to the start of each allocation
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
    /// An error that occurred while creating a layout for allocation.
    #[error("Invalid layout provided. Likely due to `num_bytes` in `allocate_align_64` being larger than isize::MAX. {0}")]
    LayoutError(#[from] LayoutError),

    /// An error that occurred while allocating memory.
    #[error(transparent)]
    AllocationFailed(#[from] AllocError),
}

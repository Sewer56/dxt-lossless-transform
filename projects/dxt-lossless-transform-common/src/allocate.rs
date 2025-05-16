//! Helper methods around memory allocation.
//!
//! Provides features like:
//! - Aligned allocations
//! - Allocations of arrays of pointers
//!
//! etc.

use core::alloc::{Layout, LayoutError};

use safe_allocator_api::allocator_api::*;
use safe_allocator_api::RawAlloc;
use thiserror::Error;

/*
/// Allocates an array of function pointers.
///
/// # Parameters
///
/// - `num_bytes`: The number of bytes to allocate in each element
///
/// # Returns
///
/// A [`RawAlloc`] containing the allocated data
pub fn allocate_array<T>(num_bytes: usize) -> Result<[*mut u8; num_bytes], AllocateError> {
    let layout = Layout::from_size_align(num_bytes, 64)?;
    RawAlloc::new(layout)?
}
    */

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

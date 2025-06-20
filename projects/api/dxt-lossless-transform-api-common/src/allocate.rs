//! Memory allocation utilities for cache line aligned allocations.
//!
//! This module provides cache line aligned memory allocation that automatically
//! selects appropriate alignment values based on the target architecture.
//!
//! ## Cache Line Sizes by Architecture
//!
//! - **x86/x86_64**: 64 bytes (Intel/AMD mainstream)
//! - **aarch64**: 64 bytes (ARM64 typical, but can vary)
//! - **Other architectures**: 64 bytes (conservative default)
//!
//! Note: Some processors may have different cache line sizes, but these values
//! represent reasonable defaults for performance-oriented code.

// Re-export types from the internal common crate
pub use dxt_lossless_transform_common::allocate::{AllocateError, FixedRawAllocArray};
use safe_allocator_api::RawAlloc;

/// Allocates data aligned to the processor's cache line size.
///
/// This function automatically selects the appropriate cache line alignment
/// based on the target architecture:
///
/// - x86/x86_64: 64-byte alignment
/// - aarch64: 64-byte alignment  
/// - Other architectures: 64-byte alignment (default)
///
/// # Parameters
///
/// - `num_bytes`: The number of bytes to allocate
///
/// # Returns
///
/// A [`RawAlloc`] containing the allocated data
///
/// # Examples
///
/// ```
/// use dxt_lossless_transform_api_common::allocate::allocate_cache_line_aligned;
///
/// // Allocate 1024 bytes aligned to cache line boundary
/// let allocation = allocate_cache_line_aligned(1024)?;
/// let ptr = allocation.as_ptr();
/// // The pointer will be aligned to 64 bytes on x86/x86_64/aarch64,
/// # Ok::<(), dxt_lossless_transform_api_common::allocate::AllocateError>(())
/// ```
pub fn allocate_cache_line_aligned(num_bytes: usize) -> Result<RawAlloc, AllocateError> {
    // Cache line sizes by architecture
    // Note: These are typical values - actual cache line sizes can vary by processor model
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    const CACHE_LINE_SIZE: usize = 64;

    #[cfg(target_arch = "aarch64")]
    const CACHE_LINE_SIZE: usize = 64;

    // Default for other architectures (RISC-V, ARM32, etc.)
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    const CACHE_LINE_SIZE: usize = 64;

    // Use the existing infrastructure from the common crate, but with our cache line size
    let layout = core::alloc::Layout::from_size_align(num_bytes, CACHE_LINE_SIZE)?;
    Ok(RawAlloc::new(layout)?)
}

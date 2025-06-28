//! File I/O operations for transform-aware file handling.
//!
//! This module provides memory-mapped file operations for transform and untransform operations
//! using `lightweight-mmap` for better performance.

mod error;
pub use error::*;

#[cfg(feature = "lightweight-mmap")]
mod lightweight_mmap_impl;

// Public API lives in there.
// If adding alternative implementation, you need to swap it out.
#[cfg(feature = "lightweight-mmap")]
pub use lightweight_mmap_impl::*;

#[cfg(not(feature = "lightweight-mmap"))]
compile_error!("The 'lightweight-mmap' feature must be enabled for file I/O operations.");

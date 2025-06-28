//! File I/O operations for transform-aware file handling.
//!
//! This module provides memory-mapped file operations for transform and untransform operations
//! using `lightweight-mmap` for better performance.

pub mod error;
pub use error::*;

#[cfg(feature = "lightweight-mmap")]
mod lightweight_mmap_impl;
#[cfg(feature = "lightweight-mmap")]
pub use lightweight_mmap_impl::*;

#[cfg(not(feature = "lightweight-mmap"))]
compile_error!("The 'lightweight-mmap' feature must be enabled for file I/O operations.");

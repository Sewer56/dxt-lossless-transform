//! BC1 utility functions.
//!
//! This module provides utility functions for BC1 data, including decoding operations.

pub use dxt_lossless_transform_bc1::util::{decode_bc1_block, decode_bc1_block_from_slice};
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

// Re-export the utility functions at the module level for convenience
// The actual implementations are in the core BC1 library

//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC3 blocks to improve compression ratio
//! by making solid color blocks and alpha values have consistent representations.
//!
//! ## BC3 Block Format
//!
//! First, let's recall the BC3 block format:
//!
//! ```text
//! Address: 0       2        8       12      16
//!          +-------+--------+  +-------+--------+
//! Data:    | A0-A1 |AI0-AI15|  | C0-C1 | I0-I15 |
//!          +-------+--------+  +-------+--------+
//! ```
//!
//! Where:
//! - `A0-A1` are two alpha endpoints (8-bit each)
//! - `AI0-AI15` are 6 bytes containing sixteen 3-bit alpha indices
//! - `C0-C1` are 16-bit RGB565 color values (2 bytes each)
//! - `I0-I15` are 4 bytes containing sixteen 2-bit color indices
//!
//! ## Normalization Rules
//!
//! The normalization process applies different rules for alpha and color components:
//!
//! ### Alpha Normalization
//!
//! When an entire block has uniform alpha, several representations are possible:
//!
//! For fully opaque blocks (`0xFF`):
//! - All 8 bytes set to `0xFF` i.e. (`0xFFFFFFFF 0xFFFFFFFF`).
//!   - Because `A0` <= `A1`, index `0xFF` is hardcoded to opaque on the decoder side.
//! - Zero alphas but indices set to `0xFF` i.e. (`0x0000FFFF 0xFFFFFFFF`).
//!
//! For all other values (including `0xFF`):
//! - `A0` set to the alpha value, everything else to `0x00` i.e. (`0xFF000000 0x00000000`).
//!   - Everything uses the alpha value from the first endpoint.
//!
//! ### Color Normalization
//!
//! For solid color blocks with clean RGB565 conversion:
//! - Set color in `C0`, zeroes in `C1` and indices
//!   - This results in a nice repetition of `0x00` across 6 bytes
//! - Or replicate color in both `C0` and `C1`, zeroes in indices
//!   - In some cases, this performs better in compression
//!
//! Note: With BC3, it's important that we put the color in `C0` because the 'alternate alpha mode' of
//! BC1 where `c0 <= c1` is unsupported; it leads to undefined behavior on some GPUs.
//!
//! ## Implementation Details
//!
//! The normalization process:
//!
//! 1. Decodes the block to get all 16 pixels with their colors and alpha values
//! 2. Checks for uniform alpha and solid colors
//! 3. Applies appropriate normalization based on the selected modes
//! 4. Writes the normalized block to the output

/// See [`super::normalize_blocks`] for the exact details.
pub mod normalize;

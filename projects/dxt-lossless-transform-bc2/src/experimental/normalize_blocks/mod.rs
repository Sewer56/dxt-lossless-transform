//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC2 blocks to improve compression ratio
//! by making solid color blocks have consistent representations.
//!
//! ## BC2 Block Format
//!
//! First, let's recall the BC2 block format:
//!
//! ```text
//! Address: 0        8       12      16
//!          +--------+-------+---------+
//! Data:    | A00-A15| C0-C1 | Indices |
//!          +--------+-------+---------+
//! ```
//!
//! Where:
//! - `A00-A15` are 8 bytes containing sixteen 4-bit alpha values (explicit alpha)
//! - `C0-C1` are 16-bit RGB565 color values (2 bytes each)
//! - `Indices` are 4 bytes containing sixteen 2-bit indices (one for each pixel in the 4x4 block)
//!
//! ## Normalization Rules
//!
//! The normalization process applies the following rules to improve compression:
//!
//! ### 1. Solid Color Blocks with Uniform Alpha
//!
//! When an entire block represents a single solid color with a clean conversion between RGBA8888
//! and RGB565, we standardize the representation:
//!
//! ```text
//! +--------+--------+--------+--------+
//! | Alpha  | Color  | 0x0000 |  0x00  |
//! +--------+--------+--------+--------+
//! ```
//!
//! We preserve the alpha values as they are, place the block's color in `Color0`, set `Color1` to zero,
//! and set all indices to zero (represented by all-zero bytes in the indices section).
//! In some cases, it's beneficial to replicate the color across `C0` and `C1` instead.
//!
//! The implementation checks for this case by:
//! 1. Decoding the block to get all 16 pixels
//! 2. Checking that all pixels have the same color (ignoring alpha)
//! 3. Verifying the color can be cleanly round-tripped through RGB565 encoding
//! 4. Constructing a new normalized block with the pattern above
//!
//! ### 2. Other Blocks
//!
//! In BC2, the explicit alpha values in the first 8 bytes already handle transparency, so there's
//! no special handling needed for transparent blocks based on color indices like in BC1.
//!
//! Unlike BC1, BC2 doesn't support the "punch-through alpha" mode (where `Color0 <= Color1`),
//! as this leads to undefined behavior on some GPUs. BC2 always uses the 4-color mode.
//!
//! ## Implementation Details
//!
//! The normalization process uses the BC2 decoder to analyze the block content, then rebuilds
//! blocks according to the rules above.
//!
//! When normalizing blocks, we:
//!
//! 1. Decode the block to get all 16 pixels with their colors and alpha values
//! 2. Check if the block contains a solid color (ignoring alpha variations)
//! 3. If it's a solid color that can be cleanly round-tripped, normalize the color part of the block
//! 4. Leave the alpha values unchanged
//! 5. Write the normalized block to the output

pub mod normalize;

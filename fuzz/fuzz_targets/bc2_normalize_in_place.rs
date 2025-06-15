#![no_main]

// This fuzz test validates the BC2 normalize_split_blocks_in_place function by checking that
// the normalized blocks decode to the same pixels as the original blocks.

use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_bc2::{
    experimental::normalize_blocks::normalize::{
        normalize_split_blocks_in_place, ColorNormalizationMode,
    },
    util::decode_bc2_block,
};
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

// Fuzz test for BC2 normalization of split blocks
// Tests that normalizing split blocks preserves visual appearance when decoded
fuzz_target!(|block: Bc2Block| {
    // Skip if c0 <= c1 as that mode is not supported on all GPUs.
    let c0_raw: u16 = u16::from_le_bytes([block.bytes[8], block.bytes[9]]);
    let c1_raw: u16 = u16::from_le_bytes([block.bytes[10], block.bytes[11]]);
    let c0 = Color565::from_raw(c0_raw);
    let c1 = Color565::from_raw(c1_raw);
    if !c0.greater_than(&c1) {
        return;
    }

    // Get a slice to the BC2 block data
    let bc2_block = &block.bytes;

    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc2_block(bc2_block.as_ptr()) };

    // Test each normalization mode
    for &color_mode in ColorNormalizationMode::all_values() {
        // Create buffers for the split block data (alpha, colors and indices)
        let mut alpha = [0u8; 8];
        let mut colors = [0u8; 4];
        let mut indices = [0u8; 4];

        // Split the BC2 block into alpha, colors and indices
        unsafe {
            copy_nonoverlapping(bc2_block.as_ptr(), alpha.as_mut_ptr(), 8);
            copy_nonoverlapping(bc2_block.as_ptr().add(8), colors.as_mut_ptr(), 4);
            copy_nonoverlapping(bc2_block.as_ptr().add(12), indices.as_mut_ptr(), 4);
        }

        // Normalize the split blocks in place with the current mode
        unsafe {
            normalize_split_blocks_in_place(
                alpha.as_ptr(),
                colors.as_mut_ptr(),
                indices.as_mut_ptr(),
                1, // Process 1 block
                color_mode,
            );
        }

        // Reconstruct the normalized block
        let mut normalized_block = [0u8; 16];
        unsafe {
            copy_nonoverlapping(alpha.as_ptr(), normalized_block.as_mut_ptr(), 8);
            copy_nonoverlapping(colors.as_ptr(), normalized_block.as_mut_ptr().add(8), 4);
            copy_nonoverlapping(indices.as_ptr(), normalized_block.as_mut_ptr().add(12), 4);
        }

        // Decode the normalized block
        let normalized_decoded = unsafe { decode_bc2_block(normalized_block.as_ptr()) };

        // Compare the two decoded blocks - they should produce the same visual output
        assert_eq!(
            original_decoded,
            normalized_decoded,
            "Normalized block (with color_mode={color_mode:?}) doesn't decode to the same pixels as the original block\n\
             Original block: {bc2_block:?}\n\
             Normalized block: {normalized_block:?}",
        );
    }
});

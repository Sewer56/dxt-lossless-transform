#![no_main]

// This fuzz test validates the BC1 normalize_split_blocks_in_place function by checking that
// the normalized blocks decode to the same pixels as the original blocks.

use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_bc1::{
    normalize_blocks::{normalize_split_blocks_in_place, ColorNormalizationMode},
    util::decode_bc1_block,
};
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc1Block {
    pub bytes: [u8; 8],
}

// Fuzz test for BC1 normalization of split blocks
// Tests that normalizing split blocks preserves visual appearance when decoded
fuzz_target!(|block: Bc1Block| {
    // Get a slice to the BC1 block data
    let bc1_block = &block.bytes;

    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc1_block(bc1_block.as_ptr()) };

    // Test each normalization mode
    for &color_mode in ColorNormalizationMode::all_values() {
        // Create buffers for the split block data (colors and indices)
        let mut colors = [0u8; 4];
        let mut indices = [0u8; 4];

        // Split the BC1 block into colors and indices
        unsafe {
            copy_nonoverlapping(bc1_block.as_ptr(), colors.as_mut_ptr(), 4);
            copy_nonoverlapping(bc1_block.as_ptr().add(4), indices.as_mut_ptr(), 4);
        }

        // Normalize the split blocks in place with the current mode
        unsafe {
            normalize_split_blocks_in_place(
                colors.as_mut_ptr(),
                indices.as_mut_ptr(),
                1, // Process 1 block
                color_mode,
            );
        }

        // Reconstruct the normalized block
        let mut normalized_block = [0u8; 8];
        unsafe {
            copy_nonoverlapping(colors.as_ptr(), normalized_block.as_mut_ptr(), 4);
            copy_nonoverlapping(indices.as_ptr(), normalized_block.as_mut_ptr().add(4), 4);
        }

        // Decode the normalized block
        let normalized_decoded = unsafe { decode_bc1_block(normalized_block.as_ptr()) };

        // Compare the two decoded blocks - they should produce the same visual output
        assert_eq!(
            original_decoded,
            normalized_decoded,
            "Normalized block (with color_mode={color_mode:?}) doesn't decode to the same pixels as the original block\n\
             Original block: {bc1_block:?}\n\
             Normalized block: {normalized_block:?}",
        );
    }
});

#![no_main]

// This fuzz test validates the BC3 normalize_split_blocks_in_place function by checking that
// the normalized blocks decode to the same pixels as the original blocks.

use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_bc3::{
    normalize_blocks::{
        normalize_split_blocks_in_place, AlphaNormalizationMode, ColorNormalizationMode,
    },
    util::decode_bc3_block,
};
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc3Block {
    pub bytes: [u8; 16],
}

// Fuzz test for BC3 normalization of split blocks
// Tests that normalizing split blocks preserves visual appearance when decoded
fuzz_target!(|block: Bc3Block| {
    // Get a slice to the BC3 block data
    let bc3_block = &block.bytes;

    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc3_block(bc3_block.as_ptr()) };

    // Test each normalization mode combination
    for &alpha_mode in AlphaNormalizationMode::all_values() {
        for &color_mode in ColorNormalizationMode::all_values() {
            // Create buffers for the split block data (alpha endpoints, alpha indices, colors and indices)
            let mut alpha_endpoints = [0u8; 2];
            let mut alpha_indices = [0u8; 6];
            let mut colors = [0u8; 4];
            let mut indices = [0u8; 4];

            // Split the BC3 block into alpha endpoints, alpha indices, colors and indices
            unsafe {
                copy_nonoverlapping(bc3_block.as_ptr(), alpha_endpoints.as_mut_ptr(), 2);
                copy_nonoverlapping(bc3_block.as_ptr().add(2), alpha_indices.as_mut_ptr(), 6);
                copy_nonoverlapping(bc3_block.as_ptr().add(8), colors.as_mut_ptr(), 4);
                copy_nonoverlapping(bc3_block.as_ptr().add(12), indices.as_mut_ptr(), 4);
            }

            // Normalize the split blocks in place with the current modes
            unsafe {
                normalize_split_blocks_in_place(
                    alpha_endpoints.as_mut_ptr(),
                    alpha_indices.as_mut_ptr(),
                    colors.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    1, // Process 1 block
                    alpha_mode,
                    color_mode,
                );
            }

            // Reconstruct the normalized block
            let mut normalized_block = [0u8; 16];
            unsafe {
                copy_nonoverlapping(alpha_endpoints.as_ptr(), normalized_block.as_mut_ptr(), 2);
                copy_nonoverlapping(
                    alpha_indices.as_ptr(),
                    normalized_block.as_mut_ptr().add(2),
                    6,
                );
                copy_nonoverlapping(colors.as_ptr(), normalized_block.as_mut_ptr().add(8), 4);
                copy_nonoverlapping(indices.as_ptr(), normalized_block.as_mut_ptr().add(12), 4);
            }

            // Decode the normalized block
            let normalized_decoded = unsafe { decode_bc3_block(normalized_block.as_ptr()) };

            // Compare the two decoded blocks - they should produce the same visual output
            assert_eq!(
                original_decoded,
                normalized_decoded,
                "Normalized block (with alpha_mode={alpha_mode:?}, color_mode={color_mode:?}) doesn't decode to the same pixels as the original block\n\
                 Original block: {bc3_block:?}\n\
                 Normalized block: {normalized_block:?}",
            );
        }
    }
});

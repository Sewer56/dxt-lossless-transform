#![no_main]

// This fuzz test validates the BC1 normalizer by checking that blocks normalized with all modes
// decode to the same pixels as the original blocks.

use core::ptr;

use dxt_lossless_transform_bc1::{
    normalize_blocks::{normalize_blocks_all_modes, ColorNormalizationMode},
    util::decode_bc1_block,
};
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc1Block {
    pub bytes: [u8; 8],
}

const MODE_COUNT: usize = ColorNormalizationMode::all_values().len();

// Fuzz test for BC1 normalization with all modes
// Tests that normalizing a block with each mode preserves its visual appearance when decoded
fuzz_target!(|block: Bc1Block| {
    // Get a slice to the BC1 block data
    let bc1_block = &block.bytes;

    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc1_block(bc1_block.as_ptr()) };

    // Get all normalization modes
    let modes = ColorNormalizationMode::all_values();

    // Create buffers for each normalization mode
    let mut normalized_blocks = [[0u8; 8]; MODE_COUNT]; // Fixed size array for the 3 modes

    // Create a fixed array of output pointers with proper initialization
    let mut output_ptrs = [ptr::null_mut(); MODE_COUNT];
    for (x, block) in normalized_blocks.iter_mut().enumerate() {
        output_ptrs[x] = block.as_mut_ptr();
    }

    // Normalize the block with all modes at once
    unsafe {
        normalize_blocks_all_modes(
            bc1_block.as_ptr(),
            &output_ptrs,
            8, // Size of BC1 block in bytes
        );
    }

    // Check each normalized block
    for (x, normalized_block) in normalized_blocks.iter().enumerate() {
        // Decode the normalized block
        let normalized_decoded = unsafe { decode_bc1_block(normalized_block.as_ptr()) };

        // Get the mode name for error messages
        let mode = modes[x];

        // Compare the two decoded blocks - they should produce the same visual output
        assert_eq!(
            original_decoded,
            normalized_decoded,
            "Normalized block with mode {mode:?} doesn't decode to the same pixels as the original block\n\
             Original block: {bc1_block:?}\n\
             Normalized block: {normalized_block:?}"
        );
    }
});

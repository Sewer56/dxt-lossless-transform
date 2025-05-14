#![no_main]

// This fuzz test validates the BC2 normalizer by checking that blocks normalized with all modes
// decode to the same pixels as the original blocks.

use core::ptr;

use dxt_lossless_transform_bc2::{
    normalize_blocks::{normalize_blocks_all_modes, ColorNormalizationMode},
    util::decode_bc2_block,
};
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

const MODE_COUNT: usize = ColorNormalizationMode::all_values().len();

// Fuzz test for BC2 normalization with all modes
// Tests that normalizing a block with each mode preserves its visual appearance when decoded
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

    // Get all normalization modes
    let modes = ColorNormalizationMode::all_values();

    // Create buffers for each normalization mode
    let mut normalized_blocks = [[0u8; 16]; MODE_COUNT]; // Fixed size array for all modes

    // Create a fixed array of output pointers with proper initialization
    let mut output_ptrs = [ptr::null_mut(); MODE_COUNT];
    for (x, block) in normalized_blocks.iter_mut().enumerate() {
        output_ptrs[x] = block.as_mut_ptr();
    }

    // Normalize the block with all modes at once
    unsafe {
        normalize_blocks_all_modes(
            bc2_block.as_ptr(),
            &mut output_ptrs,
            16, // Size of BC2 block in bytes
        );
    }

    // Check each normalized block
    for (x, normalized_block) in normalized_blocks.iter().enumerate() {
        // Decode the normalized block
        let normalized_decoded = unsafe { decode_bc2_block(normalized_block.as_ptr()) };

        // Get the mode name for error messages
        let mode = modes[x];

        // Compare the two decoded blocks - they should produce the same visual output
        assert_eq!(
            original_decoded,
            normalized_decoded,
            "Normalized block with mode {mode:?} doesn't decode to the same pixels as the original block\n\
             Original block: {bc2_block:?}\n\
             Normalized block: {normalized_block:?}",
        );
    }
});

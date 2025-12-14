#![no_main]

// This fuzz test validates the BC3 normalizer by checking that blocks normalized with all modes
// decode to the same pixels as the original blocks.

use core::ptr;

use dxt_lossless_transform_bc3::{
    experimental::normalize_blocks::normalize::{
        normalize_blocks_all_modes, AlphaNormalizationMode, ColorNormalizationMode,
    },
    util::decode_bc3_block,
};
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc3Block {
    pub bytes: [u8; 16],
}

const ALPHA_MODE_COUNT: usize = AlphaNormalizationMode::all_values().len();
const COLOR_MODE_COUNT: usize = ColorNormalizationMode::all_values().len();

// Fuzz test for BC3 normalization with all modes
// Tests that normalizing a block with each mode preserves its visual appearance when decoded
fuzz_target!(|block: Bc3Block| {
    // Skip if c0 <= c1 as that mode is not supported on all GPUs.
    let c0_raw: u16 = u16::from_le_bytes([block.bytes[8], block.bytes[9]]);
    let c1_raw: u16 = u16::from_le_bytes([block.bytes[10], block.bytes[11]]);
    let c0 = Color565::from_raw(c0_raw);
    let c1 = Color565::from_raw(c1_raw);
    if !c0.greater_than(&c1) {
        return;
    }

    // Get a slice to the BC3 block data
    let bc3_block = &block.bytes;

    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc3_block(bc3_block.as_ptr()) };

    // Get all normalization modes
    let alpha_modes = AlphaNormalizationMode::all_values();
    let color_modes = ColorNormalizationMode::all_values();

    // Create buffers for each combination of normalization modes
    let mut normalized_blocks = [[[0u8; 16]; COLOR_MODE_COUNT]; ALPHA_MODE_COUNT];

    // Create a 2D array of output pointers with proper initialization
    let mut output_ptrs = [[ptr::null_mut(); COLOR_MODE_COUNT]; ALPHA_MODE_COUNT];
    for (a_idx, a_blocks) in normalized_blocks.iter_mut().enumerate() {
        for (c_idx, block) in a_blocks.iter_mut().enumerate() {
            output_ptrs[a_idx][c_idx] = block.as_mut_ptr();
        }
    }

    // Normalize the block with all mode combinations at once
    unsafe {
        normalize_blocks_all_modes(
            bc3_block.as_ptr(),
            &output_ptrs,
            16, // Size of BC3 block in bytes
        );
    }

    // Check each normalized block
    for (a_idx, a_blocks) in normalized_blocks.iter().enumerate() {
        for (c_idx, normalized_block) in a_blocks.iter().enumerate() {
            // Decode the normalized block
            let normalized_decoded = unsafe { decode_bc3_block(normalized_block.as_ptr()) };

            // Get the mode names for error messages
            let alpha_mode = alpha_modes[a_idx];
            let color_mode = color_modes[c_idx];

            // Compare the two decoded blocks - they should produce the same visual output
            assert_eq!(
                original_decoded,
                normalized_decoded,
                "Normalized block with alpha mode {alpha_mode:?} and color mode {color_mode:?} doesn't decode to the same pixels as the original block\n\
                 Original block: {bc3_block:?}\n\
                 Normalized block: {normalized_block:?}",
            );
        }
    }
});

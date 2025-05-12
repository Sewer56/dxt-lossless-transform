#![no_main]

// This fuzz test validates the BC3 normalizer by checking that the normalized blocks decode
// to the same pixels as the original blocks.

use dxt_lossless_transform_bc3::{
    normalize_blocks::{normalize_blocks, AlphaNormalizationMode, ColorNormalizationMode},
    util::decode_bc3_block,
};
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc3Block {
    pub bytes: [u8; 16],
}

// Fuzz test for BC3 normalization
// Tests that normalizing a block preserves its visual appearance when decoded
fuzz_target!(|block: Bc3Block| {
    // Skip if c0 <= c1 for the color part, as that mode is not supported on all GPUs for BC3
    // BC3 color data starts at offset 8
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

    // Test different combinations of normalization modes
    let alpha_modes = [
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        AlphaNormalizationMode::OpaqueFillAll,
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
    ];

    let color_modes = [
        ColorNormalizationMode::Color0Only,
        ColorNormalizationMode::ReplicateColor,
    ];

    for &alpha_mode in &alpha_modes {
        for &color_mode in &color_modes {
            // Create a buffer for the normalized block
            let mut normalized_block = [0u8; 16];

            // Normalize the block with the current modes
            unsafe {
                normalize_blocks(
                    bc3_block.as_ptr(),
                    normalized_block.as_mut_ptr(),
                    16, // Size of BC3 block in bytes
                    alpha_mode,
                    color_mode,
                );
            }

            // Decode the normalized block
            let normalized_decoded = unsafe { decode_bc3_block(normalized_block.as_ptr()) };

            // Compare the two decoded blocks - they should produce the same visual output
            assert_eq!(
                original_decoded, normalized_decoded,
                "Normalized block doesn't decode to the same pixels as the original block\n\
                 Alpha mode: {alpha_mode:?}, Color mode: {color_mode:?}\n\
                 Original block: {bc3_block:?}\n\
                 Normalized block: {normalized_block:?}"
            );
        }
    }
});

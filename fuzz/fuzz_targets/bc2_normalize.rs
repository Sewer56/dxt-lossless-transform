#![no_main]

// This fuzz test validates the BC2 normalizer by checking that the normalized blocks decode 
// to the same pixels as the original blocks.

use dxt_lossless_transform_bc2::{normalize_blocks::{normalize_blocks, ColorNormalizationMode}, util::decode_bc2_block};
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

// Fuzz test for BC2 normalization
// Tests that normalizing a block preserves its visual appearance when decoded
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
    
    // Create a buffer for the normalized block
    let mut normalized_block = [0u8; 16];
    
    // Normalize the block (with repeat_colour=false)
    unsafe {
        normalize_blocks(
            bc2_block.as_ptr(),
            normalized_block.as_mut_ptr(),
            16, // Size of BC2 block in bytes
            ColorNormalizationMode::Color0Only,
        );
    }
    
    // Decode the normalized block
    let normalized_decoded = unsafe { decode_bc2_block(normalized_block.as_ptr()) };
    
    // Compare the two decoded blocks - they should produce the same visual output
    assert_eq!(
        original_decoded, 
        normalized_decoded,
        "Normalized block doesn't decode to the same pixels as the original block\n\
         Original block: {bc2_block:?}\n\
         Normalized block: {normalized_block:?}",
    );
    
    // Also test normalization with repeat_colour=true
    let mut normalized_block_repeated = [0u8; 16];
    
    // Normalize the block (with repeat_colour=true)
    unsafe {
        normalize_blocks(
            bc2_block.as_ptr(),
            normalized_block_repeated.as_mut_ptr(),
            16, // Size of BC2 block in bytes
            ColorNormalizationMode::ReplicateColor,
        );
    }
    
    // Decode the normalized block
    let normalized_repeated_decoded = unsafe { decode_bc2_block(normalized_block_repeated.as_ptr()) };
    
    // Compare with original - visual appearance should still be the same
    assert_eq!(
        original_decoded, 
        normalized_repeated_decoded,
        "Normalized block (with repeat_color=true) doesn't decode to the same pixels as the original block\n\
         Original block: {bc2_block:?}\n\
         Normalized block: {normalized_block_repeated:?}",
    );
});

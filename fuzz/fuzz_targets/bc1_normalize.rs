#![no_main]

// This fuzz test validates the BC1 normalizer by checking that the normalized blocks decode 
// to the same pixels as the original blocks.

use dxt_lossless_transform_bc1::{normalize_blocks::normalize_blocks, util::decode_bc1_block};
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc1Block {
    pub bytes: [u8; 8],
}

// Fuzz test for BC1 normalization
// Tests that normalizing a block preserves its visual appearance when decoded
fuzz_target!(|block: Bc1Block| {
    // Get a slice to the BC1 block data
    let bc1_block = &block.bytes;
    
    // Save the original block for later comparison
    let original_decoded = unsafe { decode_bc1_block(bc1_block.as_ptr()) };
    
    // Create a buffer for the normalized block
    let mut normalized_block = [0u8; 8];
    
    // Normalize the block (with repeat_colour=false)
    unsafe {
        normalize_blocks(
            bc1_block.as_ptr(),
            normalized_block.as_mut_ptr(),
            8, // Size of BC1 block in bytes
            false
        );
    }
    
    // Decode the normalized block
    let normalized_decoded = unsafe { decode_bc1_block(normalized_block.as_ptr()) };
    
    // Compare the two decoded blocks - they should produce the same visual output
    assert_eq!(
        original_decoded, 
        normalized_decoded,
        "Normalized block doesn't decode to the same pixels as the original block\n\
         Original block: {bc1_block:?}\n\
         Normalized block: {normalized_block:?}",
    );
    
    // Also test normalization with repeat_colour=true
    let mut normalized_block_repeated = [0u8; 8];
    
    // Normalize the block (with repeat_colour=true)
    unsafe {
        normalize_blocks(
            bc1_block.as_ptr(),
            normalized_block_repeated.as_mut_ptr(),
            8, // Size of BC1 block in bytes
            true
        );
    }
    
    // Decode the normalized block
    let normalized_repeated_decoded = unsafe { decode_bc1_block(normalized_block_repeated.as_ptr()) };
    
    // Compare with original - visual appearance should still be the same
    assert_eq!(
        original_decoded, 
        normalized_repeated_decoded,
        "Normalized block (with repeat_color=true) doesn't decode to the same pixels as the original block\n\
         Original block: {bc1_block:?}\n\
         Normalized block: {normalized_block_repeated:?}",
    );
});

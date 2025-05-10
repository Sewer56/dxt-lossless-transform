#![no_main]

// This fuzz test compares our BC1 decoder against our BC2 decoder for colors only, ignoring alpha.

use dxt_lossless_transform_bc1::util::decode_bc1_block;
use dxt_lossless_transform_bc2::util::decode_bc2_block;
use dxt_lossless_transform_common::color_565::Color565;
use libfuzzer_sys::{arbitrary, fuzz_target};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

// Fuzz test comparing our BC1 decoder against our BC2 decoder implementation (color component only)
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

    // Create a BC1 block from the color part of the BC2 block (last 8 bytes)
    let bc1_block = &bc2_block[8..];

    // Decode using our BC1 implementation
    let bc1_decoded = unsafe { decode_bc1_block(bc1_block.as_ptr()) };

    // Decode using our BC2 implementation (color only)
    let bc2_decoded = unsafe { decode_bc2_block(bc2_block.as_ptr()) };

    // Compare the results, ignoring alpha values
    for x in 0..16 {
        assert_eq!(
            bc1_decoded.pixels[x].r, bc2_decoded.pixels[x].r,
            "Red component mismatch at pixel {x}",
        );
        assert_eq!(
            bc1_decoded.pixels[x].g, bc2_decoded.pixels[x].g,
            "Green component mismatch at pixel {x}",
        );
        assert_eq!(
            bc1_decoded.pixels[x].b, bc2_decoded.pixels[x].b,
            "Blue component mismatch at pixel {x}",
        );
    }
});

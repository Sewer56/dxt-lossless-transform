#![no_main]

// This fuzz test compares our BC2 decoder against rgbcx-sys for colors and bcdec_rs for alpha.

use core::mem;
use dxt_lossless_transform_bc2::util::decode_bc2_block;
use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
use dxt_lossless_transform_common::{color_565::Color565, color_8888::Color8888};
use libfuzzer_sys::{arbitrary, fuzz_target};
use rgbcx_sys::root::rgbcx;

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

// Fuzz test comparing our BC2 decoder against rgbcx-sys implementation
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

    // Decode using our implementation
    let our_decoded = unsafe { decode_bc2_block(bc2_block.as_ptr()) };

    // Decode using rgbcx-sys for color and bcdec_rs for alpha
    let hybrid_decoded = hybrid_decode_bc2_to_block(bc2_block);

    // Compare the results directly
    assert_eq!(our_decoded, hybrid_decoded, "Decoded blocks don't match");
});

/// Decode BC2 block using rgbcx-sys for color (with Ideal method) and bcdec_rs for alpha
fn hybrid_decode_bc2_to_block(bc2_block: &[u8]) -> Decoded4x4Block {
    // Create buffer with properly aligned size for direct transmute
    let mut rgba_buffer = [0u8; 4 * 16]; // 4 bytes per pixel * 16 pixels

    // Create a separate buffer for bcdec_rs alpha decoding
    let mut bcdec_buffer = [0u8; 4 * 16]; // 4 bytes per pixel * 16 pixels

    // Decode using rgbcx-sys for color components with Ideal method
    unsafe {
        // The BC2 (DXT3) format has the same color block format as BC1 (DXT1)
        // We'll decode the color part (last 8 bytes) with unpack_bc1
        rgbcx::unpack_bc1(
            bc2_block.as_ptr().add(8) as *const core::ffi::c_void,
            rgba_buffer.as_mut_ptr() as *mut core::ffi::c_void,
            true, // set_alpha
            rgbcx::bc1_approx_mode::cBC1Ideal,
        );
    }

    // Decode the same block with bcdec_rs to get alpha values
    bcdec_rs::bc2(bc2_block, &mut bcdec_buffer, 4 * 4);

    // Use unsafe direct copy to Decoded4x4Block through transmute
    unsafe {
        // The memory layout is already correct - RGBA byte pattern matches Color8888 layout
        let mut pixels: [Color8888; 16] = mem::transmute(rgba_buffer);
        let bcdec_pixels: [Color8888; 16] = mem::transmute(bcdec_buffer);

        for x in 0..16 {
            pixels[x].a = bcdec_pixels[x].a;
        }
        Decoded4x4Block { pixels }
    }
}

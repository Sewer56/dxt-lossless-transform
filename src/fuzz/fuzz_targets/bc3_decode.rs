#![no_main]

// This fuzz test compares our BC3 decoder against rgbcx-sys.
// BC3 uses BC1 format for color data and BC4 compression for alpha.

use core::mem;
use dxt_lossless_transform_bc3::util::decode_bc3_block;
use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
use dxt_lossless_transform_common::{color_565::Color565, color_8888::Color8888};
use libfuzzer_sys::{arbitrary, fuzz_target};
use rgbcx_sys::root::rgbcx;

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc3Block {
    pub bytes: [u8; 16],
}

// Fuzz test comparing our BC3 decoder against rgbcx-sys and bcdec_rs implementations
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

    // Decode using our implementation
    let our_decoded = unsafe { decode_bc3_block(bc3_block.as_ptr()) };

    // Decode using reference implementations
    let reference_decoded = hybrid_decode_bc3_to_block(bc3_block);

    // Compare the results - we require exact matching with no tolerance for differences
    assert_eq!(our_decoded, reference_decoded, "Decoded blocks don't match");
});

/// Decode BC3 block using rgbcx-sys for color (with Ideal method)
fn hybrid_decode_bc3_to_block(bc3_block: &[u8]) -> Decoded4x4Block {
    // Create buffer for the rgbcx-sys decoded result
    let mut rgba_buffer = [0u8; 4 * 16]; // 4 bytes per pixel * 16 pixels

    unsafe {
        // rgbcx::unpack_bc3 decodes both color and alpha, taking the entire BC3 block
        // We've already filtered invalid inputs at the start of the fuzz test
        rgbcx::unpack_bc3(
            bc3_block.as_ptr() as *const core::ffi::c_void,
            rgba_buffer.as_mut_ptr() as *mut core::ffi::c_void,
            rgbcx::bc1_approx_mode::cBC1Ideal,
        );

        // The memory layout is already correct - RGBA byte pattern matches Color8888 layout
        // in little endian.
        let pixels: [Color8888; 16] = mem::transmute(rgba_buffer);
        Decoded4x4Block { pixels }
    }
}

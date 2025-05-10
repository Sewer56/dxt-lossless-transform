#![no_main]

// This fuzz test compares our BC2 decoder against rgbcx-sys implementation using the Ideal method.
// Extra reading: https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/

use core::mem;
use dxt_lossless_transform_bc2::util::decode_bc2_block;
use dxt_lossless_transform_common::color_8888::Color8888;
use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
use libfuzzer_sys::{arbitrary, fuzz_target};
use rgbcx_sys::root::rgbcx;

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct Bc2Block {
    pub bytes: [u8; 16],
}

// Fuzz test comparing our BC2 decoder against rgbcx-sys implementation
fuzz_target!(|block: Bc2Block| {
    // Get a slice to the BC2 block data
    let bc2_block = &block.bytes;

    // Decode using our implementation
    let our_decoded = unsafe { decode_bc2_block(bc2_block.as_ptr()) };

    // Decode using rgbcx-sys implementation with Ideal method and convert to Decoded4x4Block
    let rgbcx_decoded = rgbcx_decode_bc2_to_block(bc2_block);

    // Compare the results directly
    assert_eq!(our_decoded, rgbcx_decoded, "Decoded blocks don't match");
});

/// Decode BC2 block using rgbcx-sys with Ideal method and return it as Decoded4x4Block
fn rgbcx_decode_bc2_to_block(bc2_block: &[u8]) -> Decoded4x4Block {
    // Create buffer with properly aligned size for direct transmute
    let mut rgba_buffer = [0u8; 4 * 16]; // 4 bytes per pixel * 16 pixels

    // Decode using rgbcx-sys with Ideal method
    unsafe {
        rgbcx::unpack_bc3(
            bc2_block.as_ptr() as *const ::std::os::raw::c_void,
            rgba_buffer.as_mut_ptr() as *mut ::std::os::raw::c_void,
            rgbcx::bc1_approx_mode::cBC1Ideal,
        );
    }

    // Use unsafe direct copy to Decoded4x4Block through transmute
    unsafe {
        // The memory layout is already correct - RGBA byte pattern matches Color8888 layout
        let pixels: [Color8888; 16] = mem::transmute(rgba_buffer);
        Decoded4x4Block { pixels }
    }
}

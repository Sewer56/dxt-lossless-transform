/*
 * DXT1 Block Rearrangement Optimization Explanation
 * =================================================
 *
 * Original sequential DXT1 data layout:
 * Two 16-bit colors (4 bytes total) followed by 4 bytes of indices:
 *
 * Address: 0       4       8   8      12      16
 *          +-------+-------+   +-------+-------+
 * Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
 *          +-------+-------+   +-------+-------+
 *
 * Each 8-byte block contains:
 * - 4 bytes colors (2x RGB565 values)
 * - 4 bytes of packed indices (sixteen 2-bit indices)
 *
 * Optimized layout separates colors and indices into continuous streams:
 *
 * +-------+-------+-------+     +-------+  } Colors section
 * |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
 * +-------+-------+-------+     +-------+
 * +-------+-------+-------+     +-------+  } Indices section
 * | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
 * +-------+-------+-------+     +-------+
 *
 * This rearrangement improves compression because:
 * 1. Color endpoints tend to be spatially coherent
 * 2. Index patterns often repeat across blocks
 * 3. Separating them allows better compression of each stream
 *
 * Requirements
 * ============
 *
 * A second, separate buffer to receive the results.
 *
 * While doing it in-place is technically possible, and would be beneficial in the sense that there
 * would be improved cache locality; unfortunately, that is not possible to do in a 'single pass'
 * while maintaining the spatial coherency/order.
 *
 * Introducing a second pass meanwhile would be a performance hit.
 *
 * This is possible to do with either allocating half of a buffer, and then copying the other half back,
 * or outputting it all to a single buffer. Outputting all to single buffer is faster.
 */

/// Transform into separated color/index format preserving byte order
#[inline(always)]
pub fn transform(&self) -> Vec<u8> {
    let total_size = self.input.len();
    let num_blocks = total_size / 8;
    let colors_size = num_blocks * 4;

    let mut output = Vec::with_capacity(total_size);
    unsafe {
        output.set_len(total_size);
    }

    unsafe {
        let in_ptr = self.input.as_ptr() as *const u64;
        let out_colors = output.as_mut_ptr() as *mut u32;
        let out_indices = output.as_mut_ptr().add(colors_size) as *mut u32;

        for i in 0..num_blocks {
            // Read 8 bytes as a u64, but in a known byte order
            let bytes = (in_ptr.add(i) as *const u8)
                .cast::<[u8; 8]>()
                .read_unaligned();

            // First 4 bytes are colors, next 4 are indices
            let colors = u32::from_le_bytes(bytes[..4].try_into().unwrap());
            let indices = u32::from_le_bytes(bytes[4..].try_into().unwrap());

            // Write in little-endian byte order
            ptr::write_unaligned(out_colors.add(i), colors.to_le());
            ptr::write_unaligned(out_indices.add(i), indices.to_le());
        }
    }

    output
}

/// Transform back to normal DXT1 format preserving byte order
#[inline(always)]
pub fn untransform(input: &[u8]) -> Vec<u8> {
    let total_size = input.len();
    let num_blocks = total_size / 8;
    let colors_size = num_blocks * 4;

    let mut output = Vec::with_capacity(total_size);
    unsafe {
        output.set_len(total_size);
    }

    unsafe {
        let in_colors = input.as_ptr() as *const u32;
        let in_indices = input.as_ptr().add(colors_size) as *const u32;
        let out_ptr = output.as_mut_ptr();

        for i in 0..num_blocks {
            // Read colors and indices as little-endian u32s
            let colors = u32::from_le(ptr::read_unaligned(in_colors.add(i)));
            let indices = u32::from_le(ptr::read_unaligned(in_indices.add(i)));

            // Convert to bytes in little-endian order
            let color_bytes = colors.to_le_bytes();
            let index_bytes = indices.to_le_bytes();

            // Write the bytes in order
            let out_block = out_ptr.add(i * 8);
            ptr::copy_nonoverlapping(color_bytes.as_ptr(), out_block, 4);
            ptr::copy_nonoverlapping(index_bytes.as_ptr(), out_block.add(4), 4);
        }
    }

    output
}

use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 16 * 16) as *mut u8; // Aligned to 16 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 16-byte chunks (2x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
        // Process first 8 bytes
        let color_value1 = read_unaligned(input_ptr as *const u32);
        let index_value1 = read_unaligned(input_ptr.add(4) as *const u32);

        // Process next 8 bytes
        let color_value2 = read_unaligned(input_ptr.add(8) as *const u32);
        let index_value2 = read_unaligned(input_ptr.add(12) as *const u32);
        input_ptr = input_ptr.add(16);

        // Store to respective sections
        write_unaligned(colours_ptr, color_value1);
        write_unaligned(indices_ptr, index_value1);

        write_unaligned(colours_ptr.add(1), color_value2);
        write_unaligned(indices_ptr.add(1), index_value2);

        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }

    // Handle remaining 8-byte chunk if any
    while input_ptr < max_ptr {
        // Process remaining 8 bytes
        let color_value = read_unaligned(input_ptr as *const u32);
        let index_value = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store to respective sections
        write_unaligned(colours_ptr, color_value);
        write_unaligned(indices_ptr, index_value);

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 32 * 32) as *mut u8; // Aligned to 32 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 32-byte aligned chunks (4x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
        // Process 4 sets of 8 bytes each
        let color_value1 = read_unaligned(input_ptr as *const u32);
        let index_value1 = read_unaligned(input_ptr.add(4) as *const u32);

        let color_value2 = read_unaligned(input_ptr.add(8) as *const u32);
        let index_value2 = read_unaligned(input_ptr.add(12) as *const u32);

        let color_value3 = read_unaligned(input_ptr.add(16) as *const u32);
        let index_value3 = read_unaligned(input_ptr.add(20) as *const u32);

        let color_value4 = read_unaligned(input_ptr.add(24) as *const u32);
        let index_value4 = read_unaligned(input_ptr.add(28) as *const u32);
        input_ptr = input_ptr.add(32);

        // Store all values
        write_unaligned(colours_ptr, color_value1);
        write_unaligned(indices_ptr, index_value1);

        write_unaligned(colours_ptr.add(1), color_value2);
        write_unaligned(indices_ptr.add(1), index_value2);

        write_unaligned(colours_ptr.add(2), color_value3);
        write_unaligned(indices_ptr.add(2), index_value3);

        write_unaligned(colours_ptr.add(3), color_value4);
        write_unaligned(indices_ptr.add(3), index_value4);

        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        // Process 8 bytes
        let color_value = read_unaligned(input_ptr as *const u32);
        let index_value = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store values
        write_unaligned(colours_ptr, color_value);
        write_unaligned(indices_ptr, index_value);

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 64 * 64) as *mut u8; // Aligned to 64 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 64-byte aligned chunks (8x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
        // Process 8 sets of 8 bytes each
        let color_value1 = read_unaligned(input_ptr as *const u32);
        let index_value1 = read_unaligned(input_ptr.add(4) as *const u32);

        let color_value2 = read_unaligned(input_ptr.add(8) as *const u32);
        let index_value2 = read_unaligned(input_ptr.add(12) as *const u32);

        let color_value3 = read_unaligned(input_ptr.add(16) as *const u32);
        let index_value3 = read_unaligned(input_ptr.add(20) as *const u32);

        let color_value4 = read_unaligned(input_ptr.add(24) as *const u32);
        let index_value4 = read_unaligned(input_ptr.add(28) as *const u32);

        let color_value5 = read_unaligned(input_ptr.add(32) as *const u32);
        let index_value5 = read_unaligned(input_ptr.add(36) as *const u32);

        let color_value6 = read_unaligned(input_ptr.add(40) as *const u32);
        let index_value6 = read_unaligned(input_ptr.add(44) as *const u32);

        let color_value7 = read_unaligned(input_ptr.add(48) as *const u32);
        let index_value7 = read_unaligned(input_ptr.add(52) as *const u32);

        let color_value8 = read_unaligned(input_ptr.add(56) as *const u32);
        let index_value8 = read_unaligned(input_ptr.add(60) as *const u32);
        input_ptr = input_ptr.add(64);

        // Store all values
        write_unaligned(colours_ptr, color_value1);
        write_unaligned(indices_ptr, index_value1);

        write_unaligned(colours_ptr.add(1), color_value2);
        write_unaligned(indices_ptr.add(1), index_value2);

        write_unaligned(colours_ptr.add(2), color_value3);
        write_unaligned(indices_ptr.add(2), index_value3);

        write_unaligned(colours_ptr.add(3), color_value4);
        write_unaligned(indices_ptr.add(3), index_value4);

        write_unaligned(colours_ptr.add(4), color_value5);
        write_unaligned(indices_ptr.add(4), index_value5);

        write_unaligned(colours_ptr.add(5), color_value6);
        write_unaligned(indices_ptr.add(5), index_value6);

        write_unaligned(colours_ptr.add(6), color_value7);
        write_unaligned(indices_ptr.add(6), index_value7);

        write_unaligned(colours_ptr.add(7), color_value8);
        write_unaligned(indices_ptr.add(7), index_value8);

        colours_ptr = colours_ptr.add(8);
        indices_ptr = indices_ptr.add(8);
    }

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        // Process 8 bytes
        let color_value = read_unaligned(input_ptr as *const u32);
        let index_value = read_unaligned(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store values
        write_unaligned(colours_ptr, color_value);
        write_unaligned(indices_ptr, index_value);

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_unroll_2, "u32 unroll_2", 4)]
    #[case(u32_unroll_4, "u32 unroll_4", 8)]
    #[case(u32_unroll_8, "u32 unroll_8", 16)]
    fn portable32_transform_roundtrip(
        #[case] permute_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        run_standard_transform_roundtrip_test(permute_fn, max_blocks, impl_name);
    }
}

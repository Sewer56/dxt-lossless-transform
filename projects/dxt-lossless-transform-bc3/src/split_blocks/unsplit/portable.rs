/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr as *const u16;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u32;
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u32;

    let current_output_ptr = output_ptr;
    let alpha_byte_end_ptr = alpha_byte_in_ptr.add(len / 16);

    u32_detransform_with_separate_pointers(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        alpha_byte_end_ptr,
    );
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform_with_separate_pointers(
    mut alpha_byte_in_ptr: *const u16,
    mut alpha_bit_in_ptr: *const u16,
    mut color_byte_in_ptr: *const u32,
    mut index_byte_in_ptr: *const u32,
    mut current_output_ptr: *mut u8,
    alpha_byte_end_ptr: *const u16,
) {
    while alpha_byte_in_ptr < alpha_byte_end_ptr {
        // Alpha bytes (2 bytes)
        (current_output_ptr as *mut u16).write_unaligned(alpha_byte_in_ptr.read_unaligned());
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);

        // Alpha bits (6 bytes)
        (current_output_ptr.add(2) as *mut u16).write_unaligned(alpha_bit_in_ptr.read_unaligned());
        (current_output_ptr.add(4) as *mut u32)
            .write_unaligned((alpha_bit_in_ptr.add(1) as *const u32).read_unaligned());
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(3);

        // Color bytes (4 bytes)
        (current_output_ptr.add(8) as *mut u32).write_unaligned(color_byte_in_ptr.read_unaligned());
        color_byte_in_ptr = color_byte_in_ptr.add(1);

        // Index bytes (4 bytes)
        (current_output_ptr.add(12) as *mut u32)
            .write_unaligned(index_byte_in_ptr.read_unaligned());
        index_byte_in_ptr = index_byte_in_ptr.add(1);
        current_output_ptr = current_output_ptr.add(16);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads based on the transformed layout derived from len bytes of output.
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Set up input pointers for each section
    // Pointers are doubly sized for unroll.
    let mut alpha_byte_in_ptr = input_ptr as *const u32;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u32;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u32;

    let mut current_output_ptr = output_ptr;
    let alpha_byte_aligned_end_ptr =
        input_ptr.add((len / 16 * 2).saturating_sub(32 - 16)) as *const u32;

    while alpha_byte_in_ptr < alpha_byte_aligned_end_ptr {
        let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);

        // Alpha bytes (2 bytes). Write in 2 blocks in 1 shot.
        #[cfg(target_endian = "little")]
        {
            (current_output_ptr as *mut u16).write_unaligned(alpha_bytes as u16);
            ((current_output_ptr).add(16) as *mut u16).write_unaligned((alpha_bytes >> 16) as u16);
        }
        #[cfg(target_endian = "big")]
        {
            (current_output_ptr as *mut u16).write_unaligned((alpha_bytes >> 16) as u16);
            ((current_output_ptr).add(16) as *mut u16).write_unaligned(alpha_bytes as u16);
        }

        // Alpha bits (6 bytes)
        (current_output_ptr.add(2) as *mut u16).write_unaligned(alpha_bit_in_ptr.read_unaligned());
        (current_output_ptr.add(4) as *mut u32)
            .write_unaligned((alpha_bit_in_ptr.add(1) as *const u32).read_unaligned());
        (current_output_ptr.add(2 + 16) as *mut u16)
            .write_unaligned(alpha_bit_in_ptr.add(3).read_unaligned());
        (current_output_ptr.add(4 + 16) as *mut u32)
            .write_unaligned((alpha_bit_in_ptr.add(4) as *const u32).read_unaligned());
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(6);

        // Color bytes (4 bytes)
        (current_output_ptr.add(8) as *mut u32).write_unaligned(color_byte_in_ptr.read_unaligned());
        (current_output_ptr.add(8 + 16) as *mut u32)
            .write_unaligned(color_byte_in_ptr.add(1).read_unaligned());
        color_byte_in_ptr = color_byte_in_ptr.add(2);

        // Index bytes (4 bytes)
        (current_output_ptr.add(12) as *mut u32)
            .write_unaligned(index_byte_in_ptr.read_unaligned());
        (current_output_ptr.add(12 + 16) as *mut u32)
            .write_unaligned(index_byte_in_ptr.add(1).read_unaligned());
        index_byte_in_ptr = index_byte_in_ptr.add(2);

        current_output_ptr = current_output_ptr.add(32);
    }

    // Process remaining bytes if necessary
    let alpha_byte_end_ptr = input_ptr.add(len / 16 * 2) as *const u32;
    u32_detransform_with_separate_pointers(
        alpha_byte_in_ptr as *const u16,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        alpha_byte_end_ptr as *const u16,
    );
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u64_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Set up input pointers for each section
    // Pointers are doubly sized for unroll.
    let mut alpha_byte_in_ptr = input_ptr as *const u32;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u64;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u64;

    let mut current_output_ptr = output_ptr;
    let alpha_byte_aligned_end_ptr =
        input_ptr.add((len / 16 * 2).saturating_sub(32 - 16)) as *const u32;

    while alpha_byte_in_ptr < alpha_byte_aligned_end_ptr {
        let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
        let color_bytes = color_byte_in_ptr.read_unaligned();
        let index_bytes = index_byte_in_ptr.read_unaligned();
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);
        color_byte_in_ptr = color_byte_in_ptr.add(1);
        index_byte_in_ptr = index_byte_in_ptr.add(1);

        // Alpha bytes (2 bytes). Write in 2 blocks in 1 shot.
        #[cfg(target_endian = "little")]
        {
            (current_output_ptr as *mut u16).write_unaligned(alpha_bytes as u16);
            ((current_output_ptr).add(16) as *mut u16).write_unaligned((alpha_bytes >> 16) as u16);
        }
        #[cfg(target_endian = "big")]
        {
            (current_output_ptr as *mut u16).write_unaligned((alpha_bytes >> 16) as u16);
            ((current_output_ptr).add(16) as *mut u16).write_unaligned(alpha_bytes as u16);
        }

        // Alpha bits (6 bytes)
        (current_output_ptr.add(2) as *mut u16).write_unaligned(alpha_bit_in_ptr.read_unaligned());
        (current_output_ptr.add(4) as *mut u32)
            .write_unaligned((alpha_bit_in_ptr.add(1) as *const u32).read_unaligned());
        (current_output_ptr.add(2 + 16) as *mut u16)
            .write_unaligned(alpha_bit_in_ptr.add(3).read_unaligned());
        (current_output_ptr.add(4 + 16) as *mut u32)
            .write_unaligned((alpha_bit_in_ptr.add(4) as *const u32).read_unaligned());
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(6);

        // Color bytes (4 bytes)
        #[cfg(target_endian = "little")]
        {
            let color_index_bytes_0 =
                (color_bytes & 0xFFFFFFFF) | ((index_bytes & 0xFFFFFFFF) << 32);
            let color_index_bytes_1 = (color_bytes >> 32) | ((index_bytes >> 32) << 32);
            (current_output_ptr.add(8) as *mut u64).write_unaligned(color_index_bytes_0);
            (current_output_ptr.add(8 + 16) as *mut u64).write_unaligned(color_index_bytes_1);
        }
        #[cfg(target_endian = "big")]
        {
            let color_index_bytes_0 = (index_bytes >> 32) | ((color_bytes >> 32) << 32);
            let color_index_bytes_1 =
                (index_bytes & 0xFFFFFFFF) | ((color_bytes & 0xFFFFFFFF) << 32);
            (current_output_ptr.add(8) as *mut u64).write_unaligned(color_index_bytes_0);
            (current_output_ptr.add(8 + 16) as *mut u64).write_unaligned(color_index_bytes_1);
        }

        // Index bytes (4 bytes)
        current_output_ptr = current_output_ptr.add(32);
    }

    // Process remaining bytes if necessary
    let alpha_byte_end_ptr = input_ptr.add(len / 16 * 2) as *const u32;
    u32_detransform_with_separate_pointers(
        alpha_byte_in_ptr as *const u16,
        alpha_bit_in_ptr,
        color_byte_in_ptr as *const u32,
        index_byte_in_ptr as *const u32,
        current_output_ptr,
        alpha_byte_end_ptr as *const u16,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc3_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32_detransform, "u32")]
    #[case(u32_detransform_v2, "u32_v2")]
    #[case(u64_detransform, "u64")]
    fn test_portable_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc3_test_data(num_blocks);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform using standard implementation
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Reconstruct using the implementation being tested
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                &format!("{} (aligned)", impl_name),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(u32_detransform, "u32")]
    #[case(u32_detransform_v2, "u32_v2")]
    #[case(u64_detransform, "u64")]
    fn test_portable_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc3_test_data(num_blocks);
            
            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            unsafe {
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            }
            
            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);
            
            let mut reconstructed = vec![0u8; original.len() + 1];
            
            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed_unaligned.as_ptr().add(1),
                    reconstructed.as_mut_ptr().add(1),
                    transformed.len(),
                );
            }
            
            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{} (unaligned)", impl_name),
                num_blocks,
            );
        }
    }
}

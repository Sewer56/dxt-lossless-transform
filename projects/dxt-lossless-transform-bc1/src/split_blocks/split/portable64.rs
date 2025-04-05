#[cfg(target_endian = "big")]
#[inline(always)]
fn get_color(value: u64) -> u32 {
    (value >> 32) as u32
}

#[cfg(target_endian = "big")]
#[inline(always)]
fn get_index(value: u64) -> u32 {
    value as u32
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_color(value: u64) -> u32 {
    value as u32
}

#[cfg(target_endian = "little")]
#[inline(always)]
fn get_index(value: u64) -> u32 {
    (value >> 32) as u32
}

/// Transform into separated color/index format preserving byte order
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
#[inline(always)]
pub unsafe fn portable(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // This implementation is a good general purpose one for all vector sizes
    shift(input_ptr, output_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_ptr = input_ptr.add(len) as *mut u64;
    let mut input_ptr = input_ptr as *mut u64;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while input_ptr < max_ptr {
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colours and indices using endian-aware helpers
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 16 * 16) as *mut u64;
    let max_ptr = input_ptr.add(len) as *mut u64;
    let mut input_ptr = input_ptr as *mut u64;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 16-byte chunks (2x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
        // Load 2 blocks at once
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);
        input_ptr = input_ptr.add(2);

        // Split into colours and indices
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);
        let index1 = get_index(curr1);
        let index2 = get_index(curr2);

        // Store all colors
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;
        colours_ptr = colours_ptr.add(2);

        // Store all indices
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;
        indices_ptr = indices_ptr.add(2);
    }

    // Handle remaining 8-byte chunk if any
    while input_ptr < max_ptr {
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colours and indices using endian-aware helpers
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 32 * 32) as *mut u64;
    let max_ptr = input_ptr.add(len) as *mut u64;
    let mut input_ptr = input_ptr as *mut u64;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 32-byte aligned chunks (4x 8-byte blocks)
    while input_ptr.add(3) < max_ptr && input_ptr < max_aligned_ptr {
        // Load 4 blocks at once
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);
        let curr3 = *input_ptr.add(2);
        let curr4 = *input_ptr.add(3);
        input_ptr = input_ptr.add(4);

        // Split into colours and indices
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);
        let color3 = get_color(curr3);
        let color4 = get_color(curr4);
        let index1 = get_index(curr1);
        let index2 = get_index(curr2);
        let index3 = get_index(curr3);
        let index4 = get_index(curr4);

        // Store all colors
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;
        *colours_ptr.add(2) = color3;
        *colours_ptr.add(3) = color4;
        colours_ptr = colours_ptr.add(4);

        // Store all indices
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;
        *indices_ptr.add(2) = index3;
        *indices_ptr.add(3) = index4;
        indices_ptr = indices_ptr.add(4);
    }

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colours and indices using endian-aware helpers
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 64 * 64) as *mut u64;
    let max_ptr = input_ptr.add(len) as *mut u64;
    let mut input_ptr = input_ptr as *mut u64;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 64-byte aligned chunks (8x 8-byte blocks)
    while input_ptr.add(7) < max_ptr && input_ptr < max_aligned_ptr {
        // Load 8 blocks at once
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);
        let curr3 = *input_ptr.add(2);
        let curr4 = *input_ptr.add(3);
        let curr5 = *input_ptr.add(4);
        let curr6 = *input_ptr.add(5);
        let curr7 = *input_ptr.add(6);
        let curr8 = *input_ptr.add(7);
        input_ptr = input_ptr.add(8);

        // Split into colours and indices
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);
        let color3 = get_color(curr3);
        let color4 = get_color(curr4);
        let color5 = get_color(curr5);
        let color6 = get_color(curr6);
        let color7 = get_color(curr7);
        let color8 = get_color(curr8);

        let index1 = get_index(curr1);
        let index2 = get_index(curr2);
        let index3 = get_index(curr3);
        let index4 = get_index(curr4);
        let index5 = get_index(curr5);
        let index6 = get_index(curr6);
        let index7 = get_index(curr7);
        let index8 = get_index(curr8);

        // Store all colors
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;
        *colours_ptr.add(2) = color3;
        *colours_ptr.add(3) = color4;
        *colours_ptr.add(4) = color5;
        *colours_ptr.add(5) = color6;
        *colours_ptr.add(6) = color7;
        *colours_ptr.add(7) = color8;
        colours_ptr = colours_ptr.add(8);

        // Store all indices
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;
        *indices_ptr.add(2) = index3;
        *indices_ptr.add(3) = index4;
        *indices_ptr.add(4) = index5;
        *indices_ptr.add(5) = index6;
        *indices_ptr.add(6) = index7;
        *indices_ptr.add(7) = index8;
        indices_ptr = indices_ptr.add(8);
    }

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colours and indices using endian-aware helpers
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_with_count(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let mut num_elements = len / 8;
    let mut input_ptr = input_ptr as *mut u64;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while num_elements > 0 {
        num_elements -= 1;
        let curr = *input_ptr;

        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        *indices_ptr = index_value;

        input_ptr = input_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_with_count_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Process full 16-byte chunks
    let full_chunks = len / 16;
    let remaining_bytes = len % 16;
    let mut input_ptr = input_ptr as *mut u64;
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process full 16-byte chunks
    let mut num_elements = full_chunks;
    while num_elements > 0 {
        num_elements -= 1;

        // Load all values first
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);

        // Process all colors together
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);

        // Store all colors together
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;

        // Process all indices together
        let index1 = get_index(curr1);
        let index2 = get_index(curr2);

        // Store all indices together
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;

        input_ptr = input_ptr.add(2);
        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }

    // Process remaining 8-byte chunk if any
    if remaining_bytes > 0 {
        debug_assert_eq!(remaining_bytes, 8);

        // Process remaining 8 bytes
        let curr = *input_ptr;

        // Split into colors and indices
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store values
        *colours_ptr = color_value;
        *indices_ptr = index_value;
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_with_count_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Process full 32-byte chunks
    let full_chunks = len / 32;
    let remaining_bytes = len % 32;
    let mut input_ptr = input_ptr as *mut u64;
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process full 32-byte chunks
    let mut num_elements = full_chunks;
    while num_elements > 0 {
        num_elements -= 1;

        // Load all values first
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);
        let curr3 = *input_ptr.add(2);
        let curr4 = *input_ptr.add(3);

        // Process all colors together
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);
        let color3 = get_color(curr3);
        let color4 = get_color(curr4);

        // Store all colors together
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;
        *colours_ptr.add(2) = color3;
        *colours_ptr.add(3) = color4;

        // Process all indices together
        let index1 = get_index(curr1);
        let index2 = get_index(curr2);
        let index3 = get_index(curr3);
        let index4 = get_index(curr4);

        // Store all indices together
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;
        *indices_ptr.add(2) = index3;
        *indices_ptr.add(3) = index4;

        input_ptr = input_ptr.add(4);
        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }

    // Process remaining 8-byte chunks
    let remaining_chunks = remaining_bytes / 8;
    for _ in 0..remaining_chunks {
        // Process 8 bytes
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colors and indices
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store values
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn shift_with_count_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Process full 64-byte chunks
    let full_chunks = len / 64;
    let remaining_bytes = len % 64;
    let mut input_ptr = input_ptr as *mut u64;
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process full 64-byte chunks
    let mut num_elements = full_chunks;
    while num_elements > 0 {
        num_elements -= 1;

        // Load all values first
        let curr1 = *input_ptr;
        let curr2 = *input_ptr.add(1);
        let curr3 = *input_ptr.add(2);
        let curr4 = *input_ptr.add(3);
        let curr5 = *input_ptr.add(4);
        let curr6 = *input_ptr.add(5);
        let curr7 = *input_ptr.add(6);
        let curr8 = *input_ptr.add(7);

        // Process all colors together
        let color1 = get_color(curr1);
        let color2 = get_color(curr2);
        let color3 = get_color(curr3);
        let color4 = get_color(curr4);
        let color5 = get_color(curr5);
        let color6 = get_color(curr6);
        let color7 = get_color(curr7);
        let color8 = get_color(curr8);

        // Store all colors together
        *colours_ptr = color1;
        *colours_ptr.add(1) = color2;
        *colours_ptr.add(2) = color3;
        *colours_ptr.add(3) = color4;
        *colours_ptr.add(4) = color5;
        *colours_ptr.add(5) = color6;
        *colours_ptr.add(6) = color7;
        *colours_ptr.add(7) = color8;

        // Process all indices together
        let index1 = get_index(curr1);
        let index2 = get_index(curr2);
        let index3 = get_index(curr3);
        let index4 = get_index(curr4);
        let index5 = get_index(curr5);
        let index6 = get_index(curr6);
        let index7 = get_index(curr7);
        let index8 = get_index(curr8);

        // Store all indices together
        *indices_ptr = index1;
        *indices_ptr.add(1) = index2;
        *indices_ptr.add(2) = index3;
        *indices_ptr.add(3) = index4;
        *indices_ptr.add(4) = index5;
        *indices_ptr.add(5) = index6;
        *indices_ptr.add(6) = index7;
        *indices_ptr.add(7) = index8;

        input_ptr = input_ptr.add(8);
        colours_ptr = colours_ptr.add(8);
        indices_ptr = indices_ptr.add(8);
    }

    // Process remaining 8-byte chunks
    let remaining_chunks = remaining_bytes / 8;
    for _ in 0..remaining_chunks {
        // Process 8 bytes
        let curr = *input_ptr;
        input_ptr = input_ptr.add(1);

        // Split into colors and indices
        let color_value = get_color(curr);
        let index_value = get_index(curr);

        // Store values
        *colours_ptr = color_value;
        colours_ptr = colours_ptr.add(1);
        *indices_ptr = index_value;
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::tests::transform_with_reference_implementation;
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(portable, "64 (auto-selected)")]
    #[case(shift_unroll_8, "shift unroll-8")]
    #[case(shift_unroll_4, "shift unroll-4")]
    #[case(shift_unroll_2, "shift unroll-2")]
    #[case(shift, "shift no-unroll")]
    #[case(shift_with_count, "shift_with_count no-unroll")]
    #[case(shift_with_count_unroll_2, "shift_with_count unroll-2")]
    #[case(shift_with_count_unroll_4, "shift_with_count unroll-4")]
    #[case(shift_with_count_unroll_8, "shift_with_count unroll-8")]
    fn test_portable64_implementation(#[case] transform_fn: TransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len()];

            // Generate reference output
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            // Test the specific implementation variant provided by rstest
            output_test.as_mut_slice().fill(0);
            // Run the implementation
            unsafe {
                transform_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Verify the output
            assert_eq!(
                output_expected.as_slice(),
                output_test.as_slice(),
                "{} implementation produced different results than reference for {} blocks",
                impl_name,
                num_blocks
            );
        }
    }
}

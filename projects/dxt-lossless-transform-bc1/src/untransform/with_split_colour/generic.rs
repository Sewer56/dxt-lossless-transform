pub(crate) unsafe fn untransform_with_split_colour_generic(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    // Fallback implementation
    unsafe {
        // Initialize pointers
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        // Calculate end pointer for color0
        let color0_ptr_end = color0_ptr.add(block_count);

        while color0_ptr < color0_ptr_end {
            // Read the split color values
            let color0 = color0_ptr.read_unaligned();
            let color1 = color1_ptr.read_unaligned();
            let indices = indices_ptr.read_unaligned();

            // Write BC1 block format: [color0: u16, color1: u16, indices: u32]
            // Convert to bytes and write directly
            (output_ptr as *mut u16).write_unaligned(color0);
            (output_ptr.add(2) as *mut u16).write_unaligned(color1);
            (output_ptr.add(4) as *mut u32).write_unaligned(indices);

            // Advance all pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8);
        }
    }
}

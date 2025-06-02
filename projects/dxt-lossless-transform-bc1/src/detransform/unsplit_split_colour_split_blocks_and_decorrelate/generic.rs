use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::{
    allocate::allocate_align_64,
    color_565::{Color565, YCoCgVariant},
};

use crate::split_blocks::unsplit::unsplit_block_with_separate_pointers;
//use multiversion::multiversion;

/// Generic implementation of combined decorrelation and unsplitting for split colour-split blocks.
/// This function decorrelates the colors from their transformed/correlated form and directly
/// writes them into BC1 block format along with the indices.
///
/// # Arguments
/// * `color0_ptr` - Pointer to the array of color0 values (in transformed/correlated form)
/// * `color1_ptr` - Pointer to the array of color1 values (in transformed/correlated form)  
/// * `indices_ptr` - Pointer to the array of 4-byte indices for each block
/// * `output_ptr` - Pointer to the output buffer for BC1 blocks (8 bytes per block)
/// * `block_count` - Number of blocks to process
/// * `decorrelation_mode` - The YCoCg variant to use for decorrelation
///
/// # Safety
/// This function is unsafe because it operates on raw pointers. The caller must ensure all
/// pointers are valid and point to sufficient memory.
pub(crate) unsafe fn unsplit_split_colour_split_blocks_and_decorrelate_generic(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    // Note(sewer): I can't get good generic codegen for this at the moment,
    //              on x86, the codegen is poor, moving values in and out of SIMD registers
    //              for the decorrelation step.
    //
    //              I'm guessing the same will be the case on aarch64 (don't have a high end aarch64
    //              machine to test on). So I've opted for another solution for the time being, that
    //              uses 2 functions (which do get correctly optimized) and an intermediate work buffer.
    //
    //              This will make the function limited by memory bandwidth, due to extra copy.
    //              For x86 at least, I got custom intrinsic functions, to overcome this.

    // Allocating here has some overhead, so we'll delegate to the slower solution if under 512 bytes.
    // 64 blocks is 512 bytes
    if block_count >= 64 {
        let mut work_alloc =
            allocate_align_64(block_count * 8).expect("Failed to allocate work buffer");
        let work_ptr = work_alloc.as_mut_ptr();

        // Recorrelate colours into work area, doing the unsplit in the same process.
        Color565::recorrelate_ycocg_r_ptr_split(
            color0_ptr as *mut Color565,
            color1_ptr as *mut Color565,
            work_ptr as *mut Color565,
            block_count * 2, // 2 colour endpoints per block.
            decorrelation_mode,
        );

        // Now unsplit the colours, placing them into the final buffer
        unsplit_block_with_separate_pointers(
            work_ptr as *const u32,
            indices_ptr,
            output_ptr,
            block_count * 8,
        );
        return;
    }

    match decorrelation_mode {
        YCoCgVariant::None => unreachable_unchecked(),
        YCoCgVariant::Variant1 => unsplit_split_colour_split_blocks_and_decorrelate_variant1(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        ),
        YCoCgVariant::Variant2 => unsplit_split_colour_split_blocks_and_decorrelate_variant2(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        ),
        YCoCgVariant::Variant3 => unsplit_split_colour_split_blocks_and_decorrelate_variant3(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        ),
    }
}

/// Specialized implementation for [`YCoCgVariant::Variant1`] decorrelation.
/// This function applies variant 1 decorrelation before writing the colors.
///
/// # Arguments
/// * `color0_ptr` - Pointer to the array of color0 values (in transformed/correlated form)
/// * `color1_ptr` - Pointer to the array of color1 values (in transformed/correlated form)  
/// * `indices_ptr` - Pointer to the array of 4-byte indices for each block
/// * `output_ptr` - Pointer to the output buffer for BC1 blocks (8 bytes per block)
/// * `block_count` - Number of blocks to process
///
/// # Safety
/// This function is unsafe because it operates on raw pointers. The caller must ensure all
/// pointers are valid and point to sufficient memory.
pub(crate) unsafe fn unsplit_split_colour_split_blocks_and_decorrelate_variant1(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    unsafe {
        // Initialize pointers for iteration
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        for _ in 0..block_count {
            // Read the correlated colors and apply variant 1 decorrelation
            let decorrelated_color0 = Color565::from_raw(*color0_ptr).recorrelate_ycocg_r_var1();
            let decorrelated_color1 = Color565::from_raw(*color1_ptr).recorrelate_ycocg_r_var1();

            // Read the indices
            let indices = *indices_ptr;

            // Write the BC1 block directly: color0 (2 bytes) + color1 (2 bytes) + indices (4 bytes)
            // Colors are stored in little-endian format as u16 values
            *(output_ptr as *mut u16) = decorrelated_color0.raw_value();
            *(output_ptr.add(2) as *mut u16) = decorrelated_color1.raw_value();
            *(output_ptr.add(4) as *mut u32) = indices;

            // Advance pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8); // 8 bytes per BC1 block
        }
    }
}

/// Specialized implementation for [`YCoCgVariant::Variant2`] decorrelation.
/// This function applies variant 2 decorrelation before writing the colors.
///
/// # Arguments
/// * `color0_ptr` - Pointer to the array of color0 values (in transformed/correlated form)
/// * `color1_ptr` - Pointer to the array of color1 values (in transformed/correlated form)  
/// * `indices_ptr` - Pointer to the array of 4-byte indices for each block
/// * `output_ptr` - Pointer to the output buffer for BC1 blocks (8 bytes per block)
/// * `block_count` - Number of blocks to process
///
/// # Safety
/// This function is unsafe because it operates on raw pointers. The caller must ensure all
/// pointers are valid and point to sufficient memory.
pub(crate) unsafe fn unsplit_split_colour_split_blocks_and_decorrelate_variant2(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    unsafe {
        // Initialize pointers for iteration
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        for _ in 0..block_count {
            // Read the correlated colors and apply variant 2 decorrelation
            let decorrelated_color0 = Color565::from_raw(*color0_ptr).recorrelate_ycocg_r_var2();
            let decorrelated_color1 = Color565::from_raw(*color1_ptr).recorrelate_ycocg_r_var2();

            // Read the indices
            let indices = *indices_ptr;

            // Write the BC1 block directly: color0 (2 bytes) + color1 (2 bytes) + indices (4 bytes)
            // Colors are stored in little-endian format as u16 values
            *(output_ptr as *mut u16) = decorrelated_color0.raw_value();
            *(output_ptr.add(2) as *mut u16) = decorrelated_color1.raw_value();
            *(output_ptr.add(4) as *mut u32) = indices;

            // Advance pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8); // 8 bytes per BC1 block
        }
    }
}

/// Specialized implementation for [`YCoCgVariant::Variant3`] decorrelation.
/// This function applies variant 3 decorrelation before writing the colors.
///
/// # Arguments
/// * `color0_ptr` - Pointer to the array of color0 values (in transformed/correlated form)
/// * `color1_ptr` - Pointer to the array of color1 values (in transformed/correlated form)  
/// * `indices_ptr` - Pointer to the array of 4-byte indices for each block
/// * `output_ptr` - Pointer to the output buffer for BC1 blocks (8 bytes per block)
/// * `block_count` - Number of blocks to process
///
/// # Safety
/// This function is unsafe because it operates on raw pointers. The caller must ensure all
/// pointers are valid and point to sufficient memory.
pub(crate) unsafe fn unsplit_split_colour_split_blocks_and_decorrelate_variant3(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    unsafe {
        // Initialize pointers for iteration
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        for _ in 0..block_count {
            // Read the correlated colors and apply variant 3 decorrelation
            let decorrelated_color0 = Color565::from_raw(*color0_ptr).recorrelate_ycocg_r_var3();
            let decorrelated_color1 = Color565::from_raw(*color1_ptr).recorrelate_ycocg_r_var3();

            // Read the indices
            let indices = *indices_ptr;

            // Write the BC1 block directly: color0 (2 bytes) + color1 (2 bytes) + indices (4 bytes)
            // Colors are stored in little-endian format as u16 values
            *(output_ptr as *mut u16) = decorrelated_color0.raw_value();
            *(output_ptr.add(2) as *mut u16) = decorrelated_color1.raw_value();
            *(output_ptr.add(4) as *mut u32) = indices;

            // Advance pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8); // 8 bytes per BC1 block
        }
    }
}

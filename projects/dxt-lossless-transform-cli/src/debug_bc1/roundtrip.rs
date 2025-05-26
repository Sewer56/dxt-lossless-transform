use core::sync::atomic::{AtomicUsize, Ordering};
use std::fs;

use super::RoundtripCmd;
use crate::{
    debug::extract_blocks_from_dds, error::TransformError, util::find_all_files, DdsFilter,
};
use dxt_lossless_transform_api::DdsFormat;
use dxt_lossless_transform_bc1::{
    transform_bc1, untransform_bc1, util::decode_bc1_block, Bc1TransformDetails,
};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub(crate) fn handle_roundtrip_command(cmd: RoundtripCmd) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Testing BC1 transform/untransform roundtrip on files in: {} (recursive)",
        input_path.display()
    );

    // Collect all files recursively using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to test", entries.len());

    let files_tested = AtomicUsize::new(0);
    let files_passed = AtomicUsize::new(0);

    // Process files in parallel similar to main CLI
    entries.par_iter().for_each(|entry| {
        files_tested.fetch_add(1, Ordering::Relaxed);

        match test_bc1_roundtrip_file(entry) {
            Ok(()) => {
                println!("✓ PASSED {}", entry.path().display());
                files_passed.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                println!("✗ FAILED: {e}, {}", entry.path().display());
            }
        }
    });

    let total_tested = files_tested.load(Ordering::Relaxed);
    let total_passed = files_passed.load(Ordering::Relaxed);

    println!("\nSummary: {total_passed}/{total_tested} files passed");
    if total_passed != total_tested {
        return Err(TransformError::Debug(
            "Some roundtrip tests failed".to_string(),
        ));
    }

    Ok(())
}

fn test_bc1_roundtrip(data_ptr: *const u8, len_bytes: usize) -> Result<(), TransformError> {
    // Allocate aligned buffers
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;
    let mut roundtrip_data = allocate_align_64(len_bytes)?;

    unsafe {
        // Try all transform options
        for transform_options in Bc1TransformDetails::all_combinations() {
            // Transform the data
            transform_bc1(
                data_ptr,
                transformed_data.as_mut_ptr(),
                work_buffer.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Untransform the data back
            untransform_bc1(
                transformed_data.as_ptr(),
                roundtrip_data.as_mut_ptr(),
                work_buffer.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Compare all pixels by decoding each block
            let num_blocks = len_bytes / 8;
            for block_idx in 0..num_blocks {
                let block_offset = block_idx * 8;

                // Decode original block
                let original_block_ptr = data_ptr.add(block_offset);
                let original_decoded = decode_bc1_block(original_block_ptr);

                // Decode roundtrip block
                let roundtrip_block_ptr = roundtrip_data.as_ptr().add(block_offset);
                let roundtrip_decoded = decode_bc1_block(roundtrip_block_ptr);

                // Compare all 16 pixels in the block
                if original_decoded != roundtrip_decoded {
                    return Err(TransformError::Debug(format!(
                    "Pixel mismatch in block {block_idx} (byte offset {block_offset}). Transform/untransform is not lossless!"
                )));
                }
            }
        }
    }

    Ok(())
}

fn test_bc1_roundtrip_file(entry: &fs::DirEntry) -> Result<(), TransformError> {
    unsafe {
        extract_blocks_from_dds(
            entry,
            DdsFilter::BC1,
            |data_ptr: *const u8,
             len_bytes: usize,
             format: DdsFormat|
             -> Result<(), TransformError> {
                // Only test BC1 blocks
                if format != DdsFormat::BC1 {
                    return Ok(()); // Skip non-BC1 data
                }

                test_bc1_roundtrip(data_ptr, len_bytes)
            },
        )
    }
}

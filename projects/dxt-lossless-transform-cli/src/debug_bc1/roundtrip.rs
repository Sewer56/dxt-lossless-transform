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
    let mut roundtrip_data = allocate_align_64(len_bytes)?;

    unsafe {
        // Try all transform options
        for transform_options in Bc1TransformDetails::all_combinations() {
            // Transform the data
            transform_bc1(
                data_ptr,
                transformed_data.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Untransform the data back
            untransform_bc1(
                transformed_data.as_ptr(),
                roundtrip_data.as_mut_ptr(),
                len_bytes,
                transform_options.into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_bc1_roundtrip_on_test_file() {
        let test_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets/tests/r2-256-bc1.dds");

        // Verify the test file exists
        assert!(
            test_file_path.exists(),
            "Test file does not exist: {}",
            test_file_path.display()
        );

        // Read directory containing the test file to get a proper DirEntry
        let parent_dir = test_file_path.parent().unwrap();
        let file_name = test_file_path.file_name().unwrap();

        let dir_entry = fs::read_dir(parent_dir)
            .unwrap()
            .find(|entry| entry.as_ref().unwrap().file_name() == file_name)
            .unwrap()
            .unwrap();

        // Run the roundtrip test
        let result = test_bc1_roundtrip_file(&dir_entry);

        // Assert the test passes
        assert!(result.is_ok(), "BC1 roundtrip test failed: {result:?}");
    }
}

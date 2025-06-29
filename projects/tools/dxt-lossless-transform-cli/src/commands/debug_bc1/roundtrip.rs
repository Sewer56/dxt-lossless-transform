use core::sync::atomic::{AtomicUsize, Ordering};
use std::fs;

use super::RoundtripCmd;
use crate::{debug::extract_blocks_from_file, error::TransformError, util::find_all_files};
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::TransformFormatFilter;
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

fn test_bc1_roundtrip(data: &[u8]) -> Result<(), TransformError> {
    use dxt_lossless_transform_bc1::{
        transform_bc1_with_settings, untransform_bc1_with_settings, util::decode_bc1_block,
    };
    use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};

    let data_ptr = data.as_ptr();
    let len_bytes = data.len();

    // Test all transform combinations
    for transform_options in Bc1TransformSettings::all_combinations() {
        // Allocate buffers
        let mut transformed = allocate_align_64(len_bytes)?;
        let mut restored = allocate_align_64(len_bytes)?;

        unsafe {
            // Transform
            transform_bc1_with_settings(
                data_ptr,
                transformed.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Untransform
            untransform_bc1_with_settings(
                transformed.as_ptr(),
                restored.as_mut_ptr(),
                len_bytes,
                transform_options,
            );
        }

        // Compare all pixels by decoding each block
        let num_blocks = len_bytes / 8;
        for block_idx in 0..num_blocks {
            let block_offset = block_idx * 8;

            // Decode original block
            let original_decoded = unsafe {
                let original_block_ptr = data_ptr.add(block_offset);
                decode_bc1_block(original_block_ptr)
            };

            // Decode roundtrip block
            let roundtrip_decoded = unsafe {
                let roundtrip_block_ptr = restored.as_ptr().add(block_offset);
                decode_bc1_block(roundtrip_block_ptr)
            };

            // Compare all 16 pixels in the block
            if original_decoded != roundtrip_decoded {
                let decorr_mode = match transform_options.decorrelation_mode {
                    YCoCgVariant::None => "None",
                    YCoCgVariant::Variant1 => "YCoCg1",
                    YCoCgVariant::Variant2 => "YCoCg2",
                    YCoCgVariant::Variant3 => "YCoCg3",
                };

                let split_endpoints = if transform_options.split_colour_endpoints {
                    "Split"
                } else {
                    "NoSplit"
                };

                return Err(TransformError::Debug(format!(
                    "Pixel mismatch in block {block_idx} (byte offset {block_offset}) for transform {decorr_mode}/{split_endpoints}. Transform/untransform is not lossless!"
                )));
            }
        }
    }

    Ok(())
}

fn test_bc1_roundtrip_file(entry: &fs::DirEntry) -> Result<(), TransformError> {
    extract_blocks_from_file(
        &entry.path(),
        TransformFormatFilter::Bc1,
        |data: &[u8], _format: TransformFormat| -> Result<(), TransformError> {
            test_bc1_roundtrip(data)
        },
    )
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

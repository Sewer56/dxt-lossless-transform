use super::CompressionStatsCmd;
use crate::{
    debug::{extract_blocks_from_dds, zstd},
    error::TransformError,
    util::find_all_files,
    DdsFilter,
};
use bytesize::ByteSize;
use core::{
    fmt::Debug,
    slice,
    sync::atomic::{AtomicUsize, Ordering},
};
use dxt_lossless_transform_api::DdsFormat;
use dxt_lossless_transform_bc1::{transform_bc1, Bc1TransformDetails};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

#[derive(Debug, Clone, PartialEq, Hash, Default)]
struct CompressionStatsResult {
    file_path: String,
    original_uncompressed_size: usize,
    original_compressed_size: usize,
    all_results: Vec<TransformResult>,
}

impl CompressionStatsResult {
    fn find_best_result(&self) -> TransformResult {
        let mut best_result = TransformResult::default();
        let mut best_size = usize::MAX;

        for result in &self.all_results {
            if result.compressed_size < best_size {
                best_size = result.compressed_size;
                best_result = *result;
            }
        }

        best_result
    }

    fn find_default_result(&self) -> TransformResult {
        let default_transform = Bc1TransformDetails::default();
        self.all_results
            .iter()
            .find(|result| result.transform_options == default_transform)
            .copied()
            .expect("Default transform should always be present in all_combinations()")
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Copy)]
struct TransformResult {
    transform_options: Bc1TransformDetails,
    compressed_size: usize,
}

impl TransformResult {
    fn compression_ratio(&self, original_size: usize) -> f64 {
        self.compressed_size as f64 / original_size as f64
    }
}

pub(crate) fn handle_compression_stats_command(
    cmd: CompressionStatsCmd,
) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Analyzing BC1 compression statistics for files in: {} (recursive)",
        input_path.display()
    );
    println!("Compression level: {}", cmd.compression_level);

    // Collect all files recursively using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to analyze", entries.len());

    let files_analyzed = AtomicUsize::new(0);
    let results = Mutex::new(Vec::<CompressionStatsResult>::new());

    // Process files in parallel
    entries.par_iter().for_each(|entry| {
        match analyze_bc1_compression_file(entry, cmd.compression_level) {
            Ok(file_result) => {
                files_analyzed.fetch_add(1, Ordering::Relaxed);
                print_analyzed_file(&file_result);
                results.lock().unwrap().push(file_result);
            }
            Err(e) => {
                println!("✗ Error analyzing {}: {}", entry.path().display(), e);
            }
        }
    });

    // Print overall statistics
    let results = results.into_inner().unwrap();
    print_overall_statistics(&results);

    Ok(())
}

fn print_analyzed_file(result: &CompressionStatsResult) {
    let best_result = result.find_best_result();
    let default_result = result.find_default_result();

    let ratio_old =
        result.original_compressed_size as f64 / result.original_uncompressed_size as f64;
    let ratio_new = best_result.compression_ratio(result.original_uncompressed_size);
    let ratio_default = default_result.compression_ratio(result.original_uncompressed_size);
    let ratio_improvement = ratio_old - ratio_new;
    let default_improvement = ratio_old - ratio_default;

    println!(
        "✓ Analyzed {}: orig/default/best: {}/{}/{}, ratio orig/default/best: {:.3}/{:.3}/{:.3} (-{:.3}/-{:.3}), space saved default/best: {}/{}, best method: {}",
        get_filename(&result.file_path),               // name
        format_bytes(result.original_compressed_size), // orig
        format_bytes(default_result.compressed_size),  // default
        format_bytes(best_result.compressed_size),     // best
        ratio_old,
        ratio_default,
        ratio_new,
        default_improvement,
        ratio_improvement,
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(default_result.compressed_size)
        ), // space saved with default
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(best_result.compressed_size)
        ), // space saved with best
        format_transform_details(best_result.transform_options) // method
    );
}

fn analyze_bc1_compression_file(
    entry: &fs::DirEntry,
    compression_level: i32,
) -> Result<CompressionStatsResult, TransformError> {
    let mut file_result: CompressionStatsResult = CompressionStatsResult::default();

    unsafe {
        extract_blocks_from_dds(
            entry,
            DdsFilter::BC1,
            |data_ptr: *const u8,
             len_bytes: usize,
             format: DdsFormat|
             -> Result<(), TransformError> {
                // Only analyze BC1 blocks
                if format != DdsFormat::BC1 {
                    return Ok(()); // Skip non-BC1 data
                }

                file_result = CompressionStatsResult {
                    file_path: entry.path().display().to_string(),
                    original_uncompressed_size: len_bytes,
                    all_results: analyze_bc1_compression_transforms(
                        data_ptr,
                        len_bytes,
                        compression_level,
                    )?,
                    original_compressed_size: zstd_calc_size(
                        data_ptr,
                        len_bytes,
                        compression_level,
                    )?,
                };

                Ok(())
            },
        )?;
    }

    Ok(file_result)
}

fn analyze_bc1_compression_transforms(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
) -> Result<Vec<TransformResult>, TransformError> {
    // Allocate aligned buffers for transformations
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;

    let mut results = Vec::new();
    unsafe {
        // Test all transform combinations
        for transform_options in Bc1TransformDetails::all_combinations() {
            // Transform the data
            transform_bc1(
                data_ptr,
                transformed_data.as_mut_ptr(),
                work_buffer.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Compress the transformed data
            results.push(TransformResult {
                transform_options,
                compressed_size: zstd_calc_size(
                    transformed_data.as_ptr(),
                    len_bytes,
                    compression_level,
                )?,
            });
        }
    }

    Ok(results)
}

fn zstd_calc_size(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
) -> Result<usize, TransformError> {
    let max_compressed_size = zstd::max_alloc_for_compress_size(len_bytes);
    let mut compressed_buffer =
        unsafe { Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init() };

    Ok(unsafe {
        let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
        match zstd::compress(compression_level, original_slice, &mut compressed_buffer) {
            Ok(size) => size,
            Err(_) => {
                return Err(TransformError::Debug(
                    "Debug: Compression failed".to_owned(),
                ))
            }
        }
    })
}

/// Formats a byte count as a human-readable string
fn format_bytes(bytes: usize) -> String {
    ByteSize::b(bytes as u64).to_string()
}

/// Extracts the filename from a full path
fn get_filename(full_path: &str) -> String {
    Path::new(full_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| full_path.to_string())
}

/// Formats [`Bc1TransformDetails`] as a human-readable string  
fn format_transform_details(details: Bc1TransformDetails) -> String {
    let norm_mode = match details.color_normalization_mode {
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::None => "None",
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::Color0Only => {
            "C0Only"
        }
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::ReplicateColor => {
            "Replicate"
        }
    };

    let decorr_mode = match details.decorrelation_mode {
        dxt_lossless_transform_common::color_565::YCoCgVariant::None => "None",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1 => "YCoCg1",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2 => "YCoCg2",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3 => "YCoCg3",
    };

    let split_endpoints = if details.split_colour_endpoints {
        "Split"
    } else {
        "NoSplit"
    };

    format!("{norm_mode}/{decorr_mode}/{split_endpoints}")
}

fn print_overall_statistics(results: &[CompressionStatsResult]) {
    if results.is_empty() {
        println!("\n📊 No files analyzed.");
        return;
    }

    println!("\n📊 Overall Statistics:");
    println!("═══════════════════════════════════════════════════════════════");

    // Calculate totals
    let total_original_uncompressed: usize =
        results.iter().map(|r| r.original_uncompressed_size).sum();
    let total_original_compressed: usize = results.iter().map(|r| r.original_compressed_size).sum();
    let total_best_compressed: usize = results
        .iter()
        .map(|r| r.find_best_result().compressed_size)
        .sum();
    let total_default_compressed: usize = results
        .iter()
        .map(|r| r.find_default_result().compressed_size)
        .sum();

    // Calculate ratios
    let original_ratio = total_original_compressed as f64 / total_original_uncompressed as f64;
    let best_ratio = total_best_compressed as f64 / total_original_uncompressed as f64;
    let default_ratio = total_default_compressed as f64 / total_original_uncompressed as f64;
    let improvement_ratio = original_ratio - best_ratio;
    let default_improvement_ratio = original_ratio - default_ratio;
    let space_saved = total_original_compressed.saturating_sub(total_best_compressed);
    let default_space_saved = total_original_compressed.saturating_sub(total_default_compressed);

    // Count most common methods
    let mut method_counts = HashMap::new();
    for result in results {
        let best_result = result.find_best_result();
        *method_counts
            .entry(best_result.transform_options)
            .or_insert(0) += 1;
    }

    let most_common_method = method_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&method, &count)| (method, count));

    // Print statistics
    println!("Files analyzed: {}", results.len());
    println!(
        "Total original uncompressed size: {}",
        format_bytes(total_original_uncompressed)
    );
    println!(
        "Total original compressed size: {}",
        format_bytes(total_original_compressed)
    );
    println!(
        "Total default compressed size: {}",
        format_bytes(total_default_compressed)
    );
    println!(
        "Total best compressed size: {}",
        format_bytes(total_best_compressed)
    );
    println!();
    println!("Compression ratios:");
    println!("  Original (no transform): {original_ratio:.3}");
    println!("  Default (None/YCoCg1/Split): {default_ratio:.3}");
    println!("  Best (with transform): {best_ratio:.3}");
    println!("  Default improvement: -{default_improvement_ratio:.3}");
    println!("  Best improvement: -{improvement_ratio:.3}");
    println!();
    println!(
        "Total space saved with default: {} ({:.1}% reduction)",
        format_bytes(default_space_saved),
        (default_space_saved as f64 / total_original_compressed as f64) * 100.0
    );
    println!(
        "Total space saved with best: {} ({:.1}% reduction)",
        format_bytes(space_saved),
        (space_saved as f64 / total_original_compressed as f64) * 100.0
    );

    if let Some((method, count)) = most_common_method {
        let percentage = (count as f64 / results.len() as f64) * 100.0;
        println!(
            "Most common best method: {} ({count} files, {percentage:.1}%)",
            format_transform_details(method)
        );
    }

    println!("═══════════════════════════════════════════════════════════════");
}

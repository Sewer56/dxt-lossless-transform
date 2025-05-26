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
use std::{fs, path::Path, sync::Mutex};

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

    Ok(())
}

fn print_analyzed_file(result: &CompressionStatsResult) {
    let best_result = result.find_best_result();
    let ratio_old =
        result.original_compressed_size as f64 / result.original_uncompressed_size as f64;
    let ratio_new = best_result.compression_ratio(result.original_uncompressed_size);
    let ratio_improvement = ratio_old - ratio_new;
    println!(
        "✓ Analyzed {}: orig/new: {}/{}, ratio orig/new: {:.3}/{:.3} (-{:.3}), space saved: {}, method: {}",
        get_filename(&result.file_path),               // name
        format_bytes(result.original_compressed_size), // orig
        format_bytes(best_result.compressed_size),     // new
        ratio_old,
        ratio_new,
        ratio_improvement,
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(best_result.compressed_size)
        ), // space saved
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

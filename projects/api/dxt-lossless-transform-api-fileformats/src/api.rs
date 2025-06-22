//! High-level API functions for file format operations.
//!
//! This module provides both builder-based and direct function APIs for storing
//! transform details into various file formats. The functions follow patterns
//! similar to the CLI tool's `transform_dir_entry` for consistency.

use crate::error::{FileFormatError, FileFormatResult};
use crate::formats::bc1::EmbeddableBc1Details;
use crate::traits::file_format::FileFormatHandler;
use dxt_lossless_transform_api_common::embed::EmbeddableTransformDetails;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};
use dxt_lossless_transform_bc1_api::{
    determine_optimal_transform, transform_bc1_slice, Bc1EstimateOptionsBuilder,
};
use lightweight_mmap::handles::{ReadOnlyFileHandle, ReadWriteFileHandle};
use lightweight_mmap::mmap::{ReadOnlyMmap, ReadWriteMmap};
use std::fs;
use std::path::Path;
use std::ptr::copy_nonoverlapping;

/// Transform a BC1 file using the provided transform details.
///
/// This function transforms a BC1 file using the provided transform details,
/// with optional storage of the transform parameters in the file header.
///
/// # Parameters
///
/// - `input_path`: Path to the input file
/// - `output_path`: Path to the output file  
/// - `details`: Transform details to use
/// - `store_in_header`: Whether to store transform details in the header
///
/// # Returns
///
/// [`Ok(())`] on success, or a [`FileFormatError`] on failure.
pub fn transform_bc1_file_with_details<H: FileFormatHandler>(
    input_path: &Path,
    output_path: &Path,
    details: Bc1TransformDetails,
    store_in_header: bool,
) -> FileFormatResult<()> {
    // Open and map the input file
    let source_handle = ReadOnlyFileHandle::open(
        input_path
            .to_str()
            .ok_or_else(|| FileFormatError::InvalidFileData("Invalid input path".to_string()))?,
    )
    .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    let source_size = source_handle
        .size()
        .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))? as usize;

    let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)
        .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    // Detect and validate file format
    let file_info = H::detect_format(unsafe {
        std::slice::from_raw_parts(source_mapping.data(), source_mapping.len())
    })
    .ok_or_else(|| FileFormatError::FormatNotDetected(input_path.to_string_lossy().to_string()))?;

    let data_offset = H::get_data_offset(&file_info);

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(FileFormatError::Io)?;
    }

    // Create output file
    let target_handle = ReadWriteFileHandle::create_preallocated(
        output_path
            .to_str()
            .ok_or_else(|| FileFormatError::InvalidFileData("Invalid output path".to_string()))?,
        source_size as i64,
    )
    .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    let target_mapping = ReadWriteMmap::new(&target_handle, 0, source_size)
        .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    // Copy headers
    unsafe {
        copy_nonoverlapping(source_mapping.data(), target_mapping.data(), data_offset);
    }

    // Optionally store transform details in header
    if store_in_header {
        let embeddable_details = EmbeddableBc1Details::from(Bc1DetransformDetails::from(details));
        let header = embeddable_details.to_header();
        unsafe {
            H::embed_transform_header(target_mapping.data(), header)
                .map_err(|e| FileFormatError::Embed(e.into()))?;
        }
    }

    // Transform the data section
    let data_size = source_size - data_offset;
    if data_size % 8 != 0 {
        return Err(FileFormatError::InvalidFileData(
            "BC1 data size must be multiple of 8 bytes".to_string(),
        ));
    }

    let input_slice =
        unsafe { std::slice::from_raw_parts(source_mapping.data().add(data_offset), data_size) };
    let output_slice = unsafe {
        std::slice::from_raw_parts_mut(target_mapping.data().add(data_offset), data_size)
    };

    transform_bc1_slice(input_slice, output_slice, details)
        .map_err(|e| FileFormatError::Transform(format!("BC1 transform failed: {e:?}")))?;

    Ok(())
}

/// Transform a BC1 file with automatically determined optimal settings.
///
/// This function determines the optimal transform settings for the file and then applies
/// the transformation in a single operation.
///
/// # Parameters
///
/// - `input_path`: Path to the input file
/// - `output_path`: Path to the output file
/// - `estimator`: Size estimation operations to use
/// - `use_all_modes`: Whether to use all decorrelation modes for optimization
/// - `store_in_header`: Whether to store transform details in the header
///
/// # Returns
///
/// [`Ok(TransformResult)`] on success containing the details that were used,
/// or a [`FileFormatError`] on failure.
pub fn transform_bc1_file_with_optimal<H: FileFormatHandler, T: SizeEstimationOperations>(
    input_path: &Path,
    output_path: &Path,
    estimator: T,
    use_all_modes: bool,
    store_in_header: bool,
) -> FileFormatResult<crate::builders::bc1::TransformResult>
where
    T::Error: core::fmt::Debug,
{
    // Open and map the input file
    let source_handle = ReadOnlyFileHandle::open(
        input_path
            .to_str()
            .ok_or_else(|| FileFormatError::InvalidFileData("Invalid input path".to_string()))?,
    )
    .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    let source_size = source_handle
        .size()
        .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))? as usize;

    let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)
        .map_err(|e| FileFormatError::MemoryMapping(e.to_string()))?;

    // Detect and validate file format
    let file_info = H::detect_format(unsafe {
        std::slice::from_raw_parts(source_mapping.data(), source_mapping.len())
    })
    .ok_or_else(|| FileFormatError::FormatNotDetected(input_path.to_string_lossy().to_string()))?;

    let data_offset = H::get_data_offset(&file_info);
    let data_size = source_size - data_offset;

    if data_size % 8 != 0 {
        return Err(FileFormatError::InvalidFileData(
            "BC1 data size must be multiple of 8 bytes".to_string(),
        ));
    }

    // Extract BC1 data for analysis
    let bc1_data =
        unsafe { std::slice::from_raw_parts(source_mapping.data().add(data_offset), data_size) };

    // Determine optimal transform settings
    let options = Bc1EstimateOptionsBuilder::new()
        .use_all_decorrelation_modes(use_all_modes)
        .build(estimator);

    let optimal_details = determine_optimal_transform(bc1_data, options)
        .map_err(|e| FileFormatError::Transform(format!("Optimization failed: {e:?}")))?;

    // Now transform the file using the optimal settings
    transform_bc1_file_with_details::<H>(
        input_path,
        output_path,
        optimal_details,
        store_in_header,
    )?;

    // Return the result
    let embeddable_details =
        EmbeddableBc1Details::from(Bc1DetransformDetails::from(optimal_details));
    Ok(crate::builders::bc1::TransformResult {
        transform_details: optimal_details,
        embeddable_details,
    })
}

//! Tuning the 'Lossless Transform Utils' estimator for accurately predicting the file size after compression.
//!
//! This code provides the logic for finding optimal coefficients for the lossless-transform-utils estimator;
//! focusing on providing accurate predictions of compressed size of larger, aggregated data sets.
//!
//! # Remarks
//!
//! While this provides accurate guesses for the compressed size of typical files; it is only good at
//! estimating the compressed size of larger, aggregated data sets.
//!
//! The errors on individual files can vary widely; so I would recommend against using this code for estimating single files.

use crate::error::TransformError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub mod analyze;
pub mod tune;

/// Data collected for each file and transform combination
///
/// Note: Each file generates multiple data points (one per BC1 transform mode):
/// - Bc1Colours, Bc1DecorrelatedColours, Bc1SplitColours, Bc1SplitDecorrelatedColours
///
/// So a data point represents a (file_name, data_type) tuple, not just a file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EstimatorDataPoint {
    /// File name
    pub file_name: String,

    /// Block data size in bytes (colours only)
    pub data_size: usize,

    /// Data type (transform mode)
    pub data_type: String,

    /// Actual compressed sizes at each ZStandard level
    pub zstd_sizes: HashMap<i32, usize>,

    /// LTU parameters
    pub ltu_params: LtuParameters,
}

/// LTU (Lossless Transform Utils) parameters calculated from the data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LtuParameters {
    /// Length of the data in bytes
    pub data_len: usize,

    /// Number of LZ matches found
    pub num_lz_matches: usize,

    /// Entropy value (bits per byte)
    pub entropy: f64,
}

/// Checkpoint data for resuming interrupted tuning operations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TuningCheckpoint {
    /// Files that have been processed successfully
    pub processed_files: HashSet<String>,

    /// Accumulated data points from processed files
    pub accumulated_data: Vec<EstimatorDataPoint>,

    /// ZStandard levels being tested
    pub zstd_levels: Vec<i32>,

    /// Transform filter being used
    pub filter: String,

    /// Input directory path
    pub input_directory: String,

    /// Total number of files found initially
    pub total_files: usize,
}

/// Get the power of 2 that this size falls into
/// For example: 100 bytes -> power 7 (2^7 = 128, covers 65-128)
pub fn get_power_of_2_for_size(size: usize) -> u32 {
    if size == 0 {
        return 0;
    }

    // Find the smallest power of 2 that is >= size
    let mut power = 0;
    let mut value = 1;

    while value < size {
        power += 1;
        value *= 2;
    }

    power
}

/// Parse ZStandard levels from string
pub fn parse_zstd_levels(levels_str: &str) -> Result<Vec<i32>, TransformError> {
    if levels_str.trim().to_lowercase() == "all" {
        return Ok((1..=22).collect());
    }

    let levels: Result<Vec<i32>, _> = levels_str
        .split(',')
        .map(|s| s.trim().parse::<i32>())
        .collect();

    levels.map_err(|e| TransformError::Debug(format!("Invalid ZStandard levels format: {e}")))
}

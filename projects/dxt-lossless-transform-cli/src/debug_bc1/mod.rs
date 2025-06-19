pub(crate) mod benchmark;
pub(crate) mod benchmark_determine_best;
pub(crate) mod calc_compression_stats;
pub(crate) mod roundtrip;

use crate::debug::compression::CompressionAlgorithm;
use crate::error::TransformError;
use argh::FromArgs;
use benchmark::handle_benchmark_command;
use calc_compression_stats::handle_compression_stats_command;
use core::ptr::null_mut;
use roundtrip::handle_roundtrip_command;
use std::path::PathBuf;

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing BC1 files
#[argh(subcommand, name = "debug-bc1")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {
    Roundtrip(RoundtripCmd),
    CompressionStats(CompressionStatsCmd),
    Benchmark(BenchmarkCmd),
    BenchmarkDetermineBest(BenchmarkDetermineBestCmd),
}

#[derive(FromArgs, Debug)]
/// Test BC1 transform/untransform roundtrip on files in a directory
#[argh(subcommand, name = "test-roundtrip")]
pub struct RoundtripCmd {
    /// input directory path to test (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Compress BC1 files with all transform combinations and collect compression statistics
#[argh(subcommand, name = "compression-stats")]
pub struct CompressionStatsCmd {
    /// input directory path to analyze (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,

    /// compression level for actual compression (uses algorithm default if not specified)
    #[argh(option)]
    pub compression_level: Option<i32>,

    /// compression level for size estimation (uses algorithm default if not specified)
    #[argh(option)]
    pub estimate_compression_level: Option<i32>,

    /// compression algorithm to use for actual compression (default: zstd)
    #[argh(option, default = "CompressionAlgorithm::ZStandard")]
    pub compression_algorithm: CompressionAlgorithm,

    /// compression algorithm to use for size estimation (default: same as compression_algorithm)
    #[argh(option)]
    pub estimate_compression_algorithm: Option<CompressionAlgorithm>,

    /// enable experimental color normalization feature which saves extra space at expense of compression time
    #[argh(switch)]
    pub experimental_normalize: bool,

    /// test all decorrelation modes instead of just Variant1 and None (slower but more thorough)
    #[argh(switch)]
    pub use_all_decorrelation_modes: bool,
}

#[derive(FromArgs, Debug)]
/// Benchmark BC1 transform and detransform performance on files in a directory
#[argh(subcommand, name = "benchmark")]
pub struct BenchmarkCmd {
    /// input directory path to benchmark (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,

    /// compression level for actual compression (uses algorithm default if not specified)
    #[argh(option)]
    pub compression_level: Option<i32>,

    /// compression level for size estimation (uses algorithm default if not specified)
    #[argh(option)]
    pub estimate_compression_level: Option<i32>,

    /// number of iterations per file for performance measurement (default: 10)
    #[argh(option, default = "10")]
    pub iterations: u32,

    /// warmup iterations before measurement (default: 3)
    #[argh(option, default = "3")]
    pub warmup_iterations: u32,

    /// compression algorithm to use for actual compression (default: zstd)
    #[argh(option, default = "CompressionAlgorithm::ZStandard")]
    pub compression_algorithm: CompressionAlgorithm,

    /// compression algorithm to use for size estimation (default: same as compression_algorithm)
    #[argh(option)]
    pub estimate_compression_algorithm: Option<CompressionAlgorithm>,

    /// enable experimental color normalization feature which saves extra space at expense of compression time
    #[argh(switch)]
    pub experimental_normalize: bool,

    /// test all decorrelation modes instead of just Variant1 and None (slower but more thorough)
    #[argh(switch)]
    pub use_all_decorrelation_modes: bool,
}

#[derive(FromArgs, Debug)]
/// Benchmark BC1 determine_best_transform_details function performance on files in a directory
#[argh(subcommand, name = "benchmark-determine-best")]
pub struct BenchmarkDetermineBestCmd {
    /// input directory path to benchmark (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,

    /// compression level for size estimation (uses algorithm default if not specified)
    #[argh(option)]
    pub estimate_compression_level: Option<i32>,

    /// number of iterations per file for performance measurement (default: 2)
    #[argh(option, default = "2")]
    pub iterations: u32,

    /// warmup iterations before measurement (default: 1)
    #[argh(option, default = "1")]
    pub warmup_iterations: u32,

    /// compression algorithm to use for size estimation (default: zstd)
    #[argh(option, default = "CompressionAlgorithm::ZStandard")]
    pub estimate_compression_algorithm: CompressionAlgorithm,

    /// enable experimental color normalization feature which saves extra space at expense of compression time
    #[argh(switch)]
    pub experimental_normalize: bool,

    /// test all decorrelation modes instead of just Variant1 and None (slower but more thorough)
    #[argh(switch)]
    pub use_all_decorrelation_modes: bool,
}

// Helper functions for resolving default compression levels and algorithms

impl CompressionStatsCmd {
    /// Returns the actual compression level, using algorithm default if not specified
    pub fn get_compression_level(&self) -> i32 {
        self.compression_level
            .unwrap_or_else(|| self.compression_algorithm.default_compression_level())
    }

    /// Returns the estimate compression level, using algorithm default if not specified
    pub fn get_estimate_compression_level(&self) -> i32 {
        self.estimate_compression_level.unwrap_or_else(|| {
            self.get_estimate_compression_algorithm()
                .default_estimate_compression_level()
        })
    }

    /// Returns the estimate compression algorithm, using actual algorithm if not specified
    pub fn get_estimate_compression_algorithm(&self) -> CompressionAlgorithm {
        self.estimate_compression_algorithm
            .unwrap_or(self.compression_algorithm)
    }
}

impl BenchmarkCmd {
    /// Returns the actual compression level, using algorithm default if not specified
    pub fn get_compression_level(&self) -> i32 {
        self.compression_level
            .unwrap_or_else(|| self.compression_algorithm.default_compression_level())
    }

    /// Returns the estimate compression level, using algorithm default if not specified
    pub fn get_estimate_compression_level(&self) -> i32 {
        self.estimate_compression_level.unwrap_or_else(|| {
            self.get_estimate_compression_algorithm()
                .default_estimate_compression_level()
        })
    }

    /// Returns the estimate compression algorithm, using actual algorithm if not specified
    pub fn get_estimate_compression_algorithm(&self) -> CompressionAlgorithm {
        self.estimate_compression_algorithm
            .unwrap_or(self.compression_algorithm)
    }
}

impl BenchmarkDetermineBestCmd {
    /// Returns the estimate compression level, using algorithm default if not specified
    pub fn get_estimate_compression_level(&self) -> i32 {
        self.estimate_compression_level.unwrap_or_else(|| {
            self.estimate_compression_algorithm
                .default_estimate_compression_level()
        })
    }
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::Roundtrip(roundtrip_cmd) => handle_roundtrip_command(roundtrip_cmd),
        DebugCommands::CompressionStats(compression_stats_cmd) => {
            handle_compression_stats_command(compression_stats_cmd)
        }
        DebugCommands::Benchmark(benchmark_cmd) => handle_benchmark_command(benchmark_cmd),
        DebugCommands::BenchmarkDetermineBest(benchmark_determine_best_cmd) => {
            benchmark_determine_best::handle_benchmark_determine_best_command(
                benchmark_determine_best_cmd,
            )
        }
    }
}

/// Determines the best transform details using either the standard or experimental API
/// based on the experimental_normalize flag.
///
/// # Parameters
///
/// - `data_ptr`: Pointer to the BC1 data
/// - `len_bytes`: Length of the data in bytes
/// - `estimator`: File size estimation function
/// - `experimental_normalize`: Whether to use experimental normalization
/// - `use_all_decorrelation_modes`: Whether to test all decorrelation modes
///
/// # Returns
///
/// The best transform details for the given data
fn determine_best_transform_details_with_estimator<F>(
    data_ptr: *const u8,
    len_bytes: usize,
    estimator: F,
    experimental_normalize: bool,
    use_all_decorrelation_modes: bool,
) -> Result<dxt_lossless_transform_bc1::Bc1TransformDetails, TransformError>
where
    F: Fn(*const u8, usize) -> usize,
{
    use dxt_lossless_transform_bc1::{
        determine_optimal_transform::{determine_best_transform_details, Bc1EstimateOptions},
        experimental::normalize_blocks::determine_optimal_transform::determine_best_transform_details_with_normalization,
        Bc1TransformDetails,
    };
    let transform_options = Bc1EstimateOptions {
        file_size_estimator: estimator,
        use_all_decorrelation_modes,
    };

    unsafe {
        if experimental_normalize {
            // Use experimental API with normalization support
            let experimental_details = determine_best_transform_details_with_normalization(
                data_ptr,
                len_bytes,
                transform_options,
            )
            .map_err(|e| {
                TransformError::Debug(format!("Experimental API recommendation failed: {e}"))
            })?;

            // Convert to standard struct for compatibility
            Ok(Bc1TransformDetails {
                decorrelation_mode: experimental_details.decorrelation_mode,
                split_colour_endpoints: experimental_details.split_colour_endpoints,
            })
        } else {
            // Use standard API without normalization
            determine_best_transform_details(data_ptr, len_bytes, null_mut(), transform_options)
                .map_err(|e| TransformError::Debug(format!("API recommendation failed: {e}")))
        }
    }
}

pub(crate) mod benchmark;
pub(crate) mod benchmark_determine_best;
pub(crate) mod calc_compression_stats;
pub(crate) mod roundtrip;

use crate::debug_format::compression::CompressionAlgorithm;
use crate::debug_format::compression_size_cache::CompressionSizeCache;
use crate::debug_format::estimation::create_size_estimator;
use crate::debug_format::estimation::CachedSizeEstimator;
use crate::error::TransformError;
use argh::FromArgs;
use benchmark::handle_benchmark_command;
use calc_compression_stats::handle_compression_stats_command;
use dxt_lossless_transform_bc3::transform_bc3_auto;
use dxt_lossless_transform_bc3::Bc3EstimateSettings;
use dxt_lossless_transform_bc3::Bc3TransformSettings;
use roundtrip::handle_roundtrip_command;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing BC3 files
#[argh(subcommand, name = "debug-bc3")]
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
/// Test BC3 transform/untransform roundtrip on files in a directory
#[argh(subcommand, name = "test-roundtrip")]
pub struct RoundtripCmd {
    /// input directory path to test (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Compress BC3 files with all transform combinations and collect compression statistics
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

    /// test all decorrelation modes instead of just Variant1 and None
    /// (typical gains <0.1%; consider using estimator level closer to final compression level instead)
    #[argh(switch)]
    pub use_all_decorrelation_modes: bool,

    /// maximum file size in bytes to analyze (filters out larger files, disabled by default)
    #[argh(option)]
    pub max_size: Option<u64>,
}

#[derive(FromArgs, Debug)]
/// Benchmark BC3 transform and untransform performance on files in a directory
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

    /// test all decorrelation modes instead of just Variant1 and None
    /// (typical gains <0.1%; consider using estimator level closer to final compression level instead)
    #[argh(switch)]
    pub use_all_decorrelation_modes: bool,
}

#[derive(FromArgs, Debug)]
/// Benchmark BC3 determine_best_transform_details function performance on files in a directory
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

    /// test all decorrelation modes instead of just Variant1 and None
    /// (typical gains <0.1%; consider using estimator level closer to final compression level instead)
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

/// Determines the best transform details using the BC3 auto transform API.
///
/// # Parameters
///
/// - `data`: Slice containing the BC3 data
/// - `size_estimator`: Custom size estimator implementation
/// - `use_all_decorrelation_modes`: Whether to test all decorrelation modes
///
/// # Returns
///
/// The best transform details for the given data
pub fn determine_best_transform_details_with_custom_estimator<T>(
    data: &[u8],
    size_estimator: T,
    use_all_decorrelation_modes: bool,
) -> Result<dxt_lossless_transform_bc3::Bc3TransformSettings, TransformError>
where
    T: dxt_lossless_transform_api_common::estimate::SizeEstimationOperations,
    T::Error: core::fmt::Debug,
{
    use dxt_lossless_transform_common::allocate::allocate_align_64;

    let data_ptr = data.as_ptr();
    let len_bytes = data.len();

    let transform_options = Bc3EstimateSettings {
        size_estimator,
        use_all_decorrelation_modes,
    };

    // Allocate output buffer for the transformed data
    let mut output_buffer = allocate_align_64(len_bytes)
        .map_err(|e| TransformError::Debug(format!("Failed to allocate output buffer: {e:?}")))?;
    let output_ptr = output_buffer.as_mut_ptr();

    unsafe {
        transform_bc3_auto(data_ptr, output_ptr, len_bytes, &transform_options)
            .map_err(|e| TransformError::Debug(format!("API recommendation failed: {e:?}")))
    }
}

/// Determines the best transform details using the BC3 auto transform API. This version creates a new estimator each time.
///
/// # Parameters
///
/// - `data`: Slice containing the BC3 data
/// - `compression_level`: Compression level for the estimator
/// - `compression_algorithm`: Algorithm to use for size estimation
/// - `use_all_decorrelation_modes`: Whether to test all decorrelation modes
///
/// # Returns
///
/// The best transform details for the given data
pub fn determine_best_transform_details_with_estimator(
    data: &[u8],
    compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    use_all_decorrelation_modes: bool,
) -> Result<Bc3TransformSettings, TransformError> {
    let size_estimator = create_size_estimator(compression_algorithm, compression_level)?;
    determine_best_transform_details_with_custom_estimator(
        data,
        size_estimator,
        use_all_decorrelation_modes,
    )
}

/// Cached version of [`determine_best_transform_details_with_estimator`].
/// Will pull from cache if available, else will make a new estimate and cache the result.
///
/// # Parameters
///
/// - `data`: Slice containing the BC3 data
/// - `estimate_compression_level`: Compression level for size estimation
/// - `estimate_compression_algorithm`: Algorithm to use for size estimation
/// - `cache`: Cache for storing compression size results
/// - `use_all_decorrelation_modes`: Whether to test all decorrelation modes
///
/// # Returns
///
/// The best transform details for the given data
pub fn determine_best_transform_details_with_estimator_cached(
    data: &[u8],
    estimate_compression_level: i32,
    estimate_compression_algorithm: CompressionAlgorithm,
    use_all_decorrelation_modes: bool,
    cache: &Mutex<CompressionSizeCache>,
) -> Result<dxt_lossless_transform_bc3::Bc3TransformSettings, TransformError> {
    // Create a cached estimator that combines the algorithm estimator with caching
    let cached_estimator = CachedSizeEstimator::new(
        estimate_compression_algorithm,
        estimate_compression_level,
        cache,
    )?;

    determine_best_transform_details_with_custom_estimator(
        data,
        cached_estimator,
        use_all_decorrelation_modes,
    )
}

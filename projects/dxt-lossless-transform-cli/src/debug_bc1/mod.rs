pub(crate) mod benchmark;
pub(crate) mod calc_compression_stats;
pub(crate) mod roundtrip;

use crate::error::TransformError;
use argh::FromArgs;
use benchmark::handle_benchmark_command;
use calc_compression_stats::handle_compression_stats_command;
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

    /// compression level for zstd (default: 16)
    #[argh(option, default = "16")]
    pub compression_level: i32,

    /// compression level for zstd when using API best method estimation (default: 3)
    #[argh(option, default = "3")]
    pub estimate_compression_level: i32,
}

#[derive(FromArgs, Debug)]
/// Benchmark BC1 transform and detransform performance on files in a directory
#[argh(subcommand, name = "benchmark")]
pub struct BenchmarkCmd {
    /// input directory path to benchmark (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,

    /// compression level for zstd (default: 16)
    #[argh(option, default = "16")]
    pub compression_level: i32,

    /// compression level for zstd when using API best method estimation (default: 1)
    #[argh(option, default = "1")]
    pub estimate_compression_level: i32,

    /// number of iterations per file for performance measurement (default: 10)
    #[argh(option, default = "10")]
    pub iterations: u32,

    /// warmup iterations before measurement (default: 3)
    #[argh(option, default = "3")]
    pub warmup_iterations: u32,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::Roundtrip(roundtrip_cmd) => handle_roundtrip_command(roundtrip_cmd),
        DebugCommands::CompressionStats(compression_stats_cmd) => {
            handle_compression_stats_command(compression_stats_cmd)
        }
        DebugCommands::Benchmark(benchmark_cmd) => handle_benchmark_command(benchmark_cmd),
    }
}

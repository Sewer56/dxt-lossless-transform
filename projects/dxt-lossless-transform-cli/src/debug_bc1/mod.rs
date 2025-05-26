pub(crate) mod calc_compression_stats;
pub(crate) mod roundtrip;

use crate::error::TransformError;
use argh::FromArgs;
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

    /// compression level for zstd (default: 17)
    #[argh(option, default = "17")]
    pub compression_level: i32,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::Roundtrip(roundtrip_cmd) => handle_roundtrip_command(roundtrip_cmd),
        DebugCommands::CompressionStats(compression_stats_cmd) => {
            handle_compression_stats_command(compression_stats_cmd)
        }
    }
}

#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]

mod commands;
#[cfg(feature = "debug-format")]
mod debug_format;
mod error;
mod util;
use argh::FromArgs;
use core::error::Error;

#[derive(FromArgs, Debug)]
/// File transformation tool for DDS files
struct TopLevel {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Transform(commands::transform::TransformCmd),
    Untransform(commands::untransform::UntransformCmd),
    #[cfg(feature = "debug-bc7")]
    DebugBc7(commands::debug_bc7::DebugCmd),
    #[cfg(feature = "debug-bc1")]
    DebugBc1(commands::debug_bc1::DebugCmd),
    #[cfg(feature = "debug-bc2")]
    DebugBc2(commands::debug_bc2::DebugCmd),
    #[cfg(feature = "debug-endian")]
    DebugEndian(commands::debug_endian::DebugEndianCmd),
    #[cfg(feature = "debug-endian")]
    DebugEndianTransform(commands::debug_endian::DebugEndianTransformCmd),
    #[cfg(feature = "debug-endian")]
    DebugEndianUntransform(commands::debug_endian::DebugEndianUntransformCmd),
    #[cfg(feature = "debug-format")]
    DebugFormatAnalysis(commands::debug_format_analysis::DebugFormatAnalysisCmd),
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: TopLevel = argh::from_env();

    match cli.command {
        Commands::Transform(cmd) => {
            commands::transform::handle_transform_command(cmd)?;
        }
        Commands::Untransform(cmd) => {
            commands::untransform::handle_untransform_command(cmd)?;
        }
        #[cfg(feature = "debug-bc7")]
        Commands::DebugBc7(cmd) => {
            commands::debug_bc7::handle_debug_command(cmd)?;
        }
        #[cfg(feature = "debug-bc1")]
        Commands::DebugBc1(cmd) => {
            commands::debug_bc1::handle_debug_command(cmd)?;
        }
        #[cfg(feature = "debug-bc2")]
        Commands::DebugBc2(cmd) => {
            commands::debug_bc2::handle_debug_command(cmd)?;
        }
        #[cfg(feature = "debug-endian")]
        Commands::DebugEndian(cmd) => {
            commands::debug_endian::handle_debug_command(cmd)?;
        }
        #[cfg(feature = "debug-endian")]
        Commands::DebugEndianTransform(cmd) => {
            commands::debug_endian::handle_debug_endian_transform_command(cmd)?;
        }
        #[cfg(feature = "debug-endian")]
        Commands::DebugEndianUntransform(cmd) => {
            commands::debug_endian::handle_debug_endian_untransform_command(cmd)?;
        }
        #[cfg(feature = "debug-format")]
        Commands::DebugFormatAnalysis(cmd) => {
            commands::debug_format_analysis::handle_debug_format_analysis_command(cmd)?;
        }
    }

    Ok(())
}

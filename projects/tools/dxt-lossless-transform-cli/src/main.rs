#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]

#[cfg(feature = "debug")]
mod debug;
#[cfg(feature = "debug-bc1")]
mod debug_bc1;
#[cfg(feature = "debug-bc7")]
mod debug_bc7;

mod detransform;
mod error;
mod transform;
mod util;
use argh::FromArgs;
use core::error::Error;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone)]
pub enum DdsFilter {
    BC1,
    BC2,
    BC3,
    BC7,
    All,
}

// Implement FromStr to allow parsing from command line arguments
impl FromStr for DdsFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bc1" => Ok(DdsFilter::BC1),
            "bc2" => Ok(DdsFilter::BC2),
            "bc3" => Ok(DdsFilter::BC3),
            "bc7" => Ok(DdsFilter::BC7),
            "all" => Ok(DdsFilter::All),
            _ => Err(format!(
                "Invalid DDS type: {s}. Valid types are: bc1, bc2, bc3, bc7, all"
            )),
        }
    }
}

#[derive(FromArgs, Debug)]
/// File transformation tool for DDS files
struct TopLevel {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Transform(transform::TransformCmd),
    Detransform(detransform::DetransformCmd),
    #[cfg(feature = "debug-bc7")]
    DebugBc7(debug_bc7::DebugCmd),
    #[cfg(feature = "debug-bc1")]
    DebugBc1(debug_bc1::DebugCmd),
}

pub fn canonicalize_cli_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);

    // If path doesn't exist, create it
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Now we can canonicalize it
    fs::canonicalize(path).map_err(|e| format!("Invalid path: {e}"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: TopLevel = argh::from_env();

    match cli.command {
        Commands::Transform(cmd) => {
            transform::handle_transform_command(cmd)?;
        }
        Commands::Detransform(cmd) => {
            detransform::handle_detransform_command(cmd)?;
        }
        #[cfg(feature = "debug-bc7")]
        Commands::DebugBc7(cmd) => {
            debug_bc7::handle_debug_command(cmd)?;
        }
        #[cfg(feature = "debug-bc1")]
        Commands::DebugBc1(cmd) => {
            debug_bc1::handle_debug_command(cmd)?;
        }
    }

    Ok(())
}

//! Format analysis command for analyzing files recursively and grouping by TransformFormat.

use crate::util::{all_handlers, find_all_files};
use argh::FromArgs;
use bytesize::ByteSize;
use core::error::Error;
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::{get_transform_format, TransformFormatFilter};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "analyze-formats")]
/// Analyze files recursively and group by their [`TransformFormat`]
pub struct DebugFormatAnalysisCmd {
    #[argh(positional)]
    /// directory to analyze recursively
    pub directory: PathBuf,
}

#[derive(Default)]
struct FormatStats {
    count: usize,
    total_size: u64,
}

/// Format key that explicitly lists all [`TransformFormat`] variants for HashMap compatibility.
/// Unknown is first so [`TransformFormat`] conversion is just format as u8 + 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum FormatKey {
    /// Unknown or unsupported format
    Unknown = 0,
    /// BC1 format transform (TransformFormat::Bc1 = 0x00 -> FormatKey = 1)
    Bc1 = 1,
    /// BC2 format transform (TransformFormat::Bc2 = 0x01 -> FormatKey = 2)
    Bc2 = 2,
    /// BC3 format transform (TransformFormat::Bc3 = 0x02 -> FormatKey = 3)
    Bc3 = 3,
    /// BC7 format transform (TransformFormat::Bc7 = 0x03 -> FormatKey = 4)
    Bc7 = 4,
    /// BC6H format transform (TransformFormat::Bc6H = 0x04 -> FormatKey = 5)
    Bc6H = 5,
    /// RGBA8888 format transform (TransformFormat::Rgba8888 = 0x05 -> FormatKey = 6)
    Rgba8888 = 6,
    /// BGRA8888 format transform (TransformFormat::Bgra8888 = 0x06 -> FormatKey = 7)
    Bgra8888 = 7,
    /// BGR888 format transform (TransformFormat::Bgr888 = 0x07 -> FormatKey = 8)
    Bgr888 = 8,
}

impl From<Option<TransformFormat>> for FormatKey {
    fn from(format: Option<TransformFormat>) -> Self {
        match format {
            Some(f) => {
                // Direct conversion: TransformFormat as u8 + 1 (since Unknown = 0)
                match f {
                    TransformFormat::Bc1 => Self::Bc1,           // 0x00 -> 1
                    TransformFormat::Bc2 => Self::Bc2,           // 0x01 -> 2
                    TransformFormat::Bc3 => Self::Bc3,           // 0x02 -> 3
                    TransformFormat::Bc7 => Self::Bc7,           // 0x03 -> 4
                    TransformFormat::Bc6H => Self::Bc6H,         // 0x04 -> 5
                    TransformFormat::Rgba8888 => Self::Rgba8888, // 0x05 -> 6
                    TransformFormat::Bgra8888 => Self::Bgra8888, // 0x06 -> 7
                    TransformFormat::Bgr888 => Self::Bgr888,     // 0x07 -> 8
                    _ => Self::Unknown, // Handle any future variants as unknown
                }
            }
            None => Self::Unknown,
        }
    }
}

impl FormatKey {
    /// Get a display name for this format
    fn display_name(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Bc1 => "Bc1",
            Self::Bc2 => "Bc2",
            Self::Bc3 => "Bc3",
            Self::Bc7 => "Bc7",
            Self::Bc6H => "Bc6H",
            Self::Rgba8888 => "Rgba8888",
            Self::Bgra8888 => "Bgra8888",
            Self::Bgr888 => "Bgr888",
        }
    }
}

/// Handle the debug format analysis command
pub fn handle_debug_format_analysis_command(
    cmd: DebugFormatAnalysisCmd,
) -> Result<(), Box<dyn Error>> {
    let base_dir = &cmd.directory;

    if !base_dir.exists() {
        return Err(format!("Directory does not exist: {}", base_dir.display()).into());
    }

    if !base_dir.is_dir() {
        return Err(format!("Path is not a directory: {}", base_dir.display()).into());
    }

    println!("Analyzing files in: {}", base_dir.display());
    println!();

    // Use HashMap for automatic handling of any format types
    let mut stats: HashMap<FormatKey, FormatStats> = HashMap::new();
    let mut processed_files = 0;

    // Find all files using the existing utility function
    let mut entries = Vec::new();
    find_all_files(base_dir, &mut entries)?;

    let total_files = entries.len();

    for entry in entries {
        let file_path = entry.path();

        // Get relative path for display
        let relative_path = match file_path.strip_prefix(base_dir) {
            Ok(rel) => rel,
            Err(_) => file_path.as_path(),
        };

        // Try to detect format using all available handlers
        let detected_format = detect_file_format(&file_path);

        // Print individual file result
        match detected_format {
            Some(format) => {
                println!("{:?}: {}", format, relative_path.display());
            }
            None => {
                println!("Unknown: {}", relative_path.display());
            }
        }

        // Get file size
        let file_size = match file_path.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => 0, // If we can't get size, count as 0 bytes
        };

        // Update statistics
        let format_key = FormatKey::from(detected_format);
        let entry = stats.entry(format_key).or_default();
        entry.count += 1;
        entry.total_size += file_size;

        processed_files += 1;
    }

    // Print summary
    println!();
    println!("=== SUMMARY ===");

    // Calculate total size for summary
    let total_size: u64 = stats.values().map(|s| s.total_size).sum();

    println!("Total files processed: {processed_files}/{total_files}");
    println!("Total size: {}", ByteSize(total_size));
    println!();

    // Sort the statistics by format key for consistent output
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by_key(|(format_key, _)| *format_key);

    for (format_key, format_stats) in sorted_stats {
        let count_percentage = if processed_files > 0 {
            (format_stats.count as f64 / processed_files as f64) * 100.0
        } else {
            0.0
        };

        let size_percentage = if total_size > 0 {
            (format_stats.total_size as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        println!(
            "{}: {} files ({count_percentage:.1}%), {} ({size_percentage:.1}%)",
            format_key.display_name(),
            format_stats.count,
            ByteSize(format_stats.total_size)
        );
    }

    Ok(())
}

/// Detect the transform format of a file using available handlers
fn detect_file_format(file_path: &Path) -> Option<TransformFormat> {
    // Use the existing all_handlers function properly
    match get_transform_format(file_path, &all_handlers(), TransformFormatFilter::All) {
        Ok(Some(format)) => Some(format),
        Ok(None) => None, // No handler supports this format
        Err(_) => None,   // Handler failed to parse this file or I/O error
    }
}

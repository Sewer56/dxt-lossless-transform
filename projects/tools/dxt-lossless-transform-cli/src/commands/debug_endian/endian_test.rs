use crate::commands::debug_endian::{file_compare, EndianTestResult};
use crate::error::TransformError;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// PowerPC big-endian target for cross-compilation
const BIG_ENDIAN_TARGET: &str = "powerpc64-unknown-linux-gnu";

/// Run a single endianness test for a specific file and preset
pub fn run_single_test(file: &str, preset: &str) -> Result<EndianTestResult, TransformError> {
    // Create a unique temporary directory for this test
    let temp_dir = create_temp_dir()?;
    let little_endian_dir = temp_dir.join("little");
    let big_endian_dir = temp_dir.join("big");

    // Create subdirectories
    fs::create_dir_all(&little_endian_dir).map_err(|e| {
        TransformError::Debug(format!("Failed to create little endian directory: {e}"))
    })?;
    fs::create_dir_all(&big_endian_dir).map_err(|e| {
        TransformError::Debug(format!("Failed to create big endian directory: {e}"))
    })?;

    // Extract format from filename
    let format = extract_format_from_filename(file);

    let mut result = EndianTestResult {
        format: format.clone(),
        preset: preset.to_string(),
        transform_success: false,
        untransform_success: false,
        files_identical: false,
    };

    // Run transform tests
    let transform_success =
        run_transform_comparison(file, preset, &little_endian_dir, &big_endian_dir)?;
    result.transform_success = transform_success;

    // Run untransform tests (only if transform succeeded)
    if transform_success {
        let untransform_success =
            run_untransform_comparison(file, preset, &little_endian_dir, &big_endian_dir)?;
        result.untransform_success = untransform_success;
        result.files_identical = transform_success && untransform_success;
    }

    // Clean up temporary directory
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        eprintln!(
            "Warning: Failed to clean up temporary directory {}: {}",
            temp_dir.display(),
            e
        );
    }

    Ok(result)
}

/// Run transform operation comparison between little and big endian
#[allow(unused_variables)]
fn run_transform_comparison(
    file: &str,
    preset: &str,
    little_endian_dir: &Path,
    big_endian_dir: &Path,
) -> Result<bool, TransformError> {
    let assets_dir = get_assets_test_dir()?;

    // Run transform on native (little-endian) target
    let transform_le = run_cli_command(
        &[
            "transform",
            "--input",
            assets_dir.to_str().unwrap(),
            "--output",
            little_endian_dir.to_str().unwrap(),
            "--preset",
            preset,
        ],
        None,
    )?;

    // Run transform using cross (big-endian target)
    let transform_be = run_cross_command(&[
        "run",
        "--target",
        BIG_ENDIAN_TARGET,
        "--bin",
        "dxt-lossless-transform-cli",
        "--",
        "transform",
        "--input",
        assets_dir.to_str().unwrap(),
        "--output",
        big_endian_dir.to_str().unwrap(),
        "--preset",
        preset,
    ])?;

    if !transform_le || !transform_be {
        return Ok(false);
    }

    // Compare output directories
    file_compare::compare_directories(little_endian_dir, big_endian_dir)
}

/// Run untransform operation comparison between little and big endian
#[allow(unused_variables)]
fn run_untransform_comparison(
    file: &str,
    preset: &str,
    little_endian_dir: &Path,
    big_endian_dir: &Path,
) -> Result<bool, TransformError> {
    // Create subdirectories for untransform outputs
    let le_untransform_dir = little_endian_dir.join("untransformed");
    let be_untransform_dir = big_endian_dir.join("untransformed");

    fs::create_dir_all(&le_untransform_dir).map_err(|e| {
        TransformError::Debug(format!("Failed to create LE untransform directory: {e}"))
    })?;
    fs::create_dir_all(&be_untransform_dir).map_err(|e| {
        TransformError::Debug(format!("Failed to create BE untransform directory: {e}"))
    })?;

    // Run untransform on the previously transformed files
    let untransform_le = run_cli_command(
        &[
            "untransform",
            "--input",
            little_endian_dir.to_str().unwrap(),
            "--output",
            le_untransform_dir.to_str().unwrap(),
        ],
        None,
    )?;

    let untransform_be = run_cross_command(&[
        "run",
        "--target",
        BIG_ENDIAN_TARGET,
        "--bin",
        "dxt-lossless-transform-cli",
        "--",
        "untransform",
        "--input",
        big_endian_dir.to_str().unwrap(),
        "--output",
        be_untransform_dir.to_str().unwrap(),
    ])?;

    if !untransform_le || !untransform_be {
        return Ok(false);
    }

    // Compare untransform output directories
    file_compare::compare_directories(&le_untransform_dir, &be_untransform_dir)
}

/// Run a CLI command using cargo run
fn run_cli_command(args: &[&str], working_dir: Option<&Path>) -> Result<bool, TransformError> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "dxt-lossless-transform-cli",
        "--features",
        "debug-endian",
        "--",
    ]);
    cmd.args(args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .map_err(|e| TransformError::Debug(format!("Failed to execute cargo command: {e}")))?;

    if !output.status.success() {
        eprintln!("Command failed with output:");
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output.status.success())
}

/// Run a CLI command using cross
fn run_cross_command(args: &[&str]) -> Result<bool, TransformError> {
    let mut cmd = Command::new("cross");
    cmd.args(args);

    let output = cmd
        .output()
        .map_err(|e| TransformError::Debug(format!("Failed to execute cross command: {e}")))?;

    if !output.status.success() {
        eprintln!("Cross command failed with output:");
        eprintln!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(output.status.success())
}

/// Create a unique temporary directory
fn create_temp_dir() -> Result<PathBuf, TransformError> {
    let base_temp = env::temp_dir();
    let unique_name = format!("dxt-endian-test-{}", std::process::id());
    let temp_dir = base_temp.join(unique_name);

    fs::create_dir_all(&temp_dir)
        .map_err(|e| TransformError::Debug(format!("Failed to create temporary directory: {e}")))?;

    Ok(temp_dir)
}

/// Get the path to the assets/tests directory
fn get_assets_test_dir() -> Result<PathBuf, TransformError> {
    // Navigate from the current working directory to find assets/tests
    let cwd = env::current_dir()
        .map_err(|e| TransformError::Debug(format!("Cannot get current directory: {e}")))?;

    let assets_dir = cwd.join("assets").join("tests");

    if !assets_dir.exists() {
        return Err(TransformError::Debug(format!(
            "Assets test directory not found at: {}. Make sure you're running from the project root.",
            assets_dir.display()
        )));
    }

    Ok(assets_dir)
}

/// Extract format name from filename (e.g., "r2-256-bc1.dds" -> "BC1")
fn extract_format_from_filename(filename: &str) -> String {
    if filename.contains("bc1") {
        "BC1".to_string()
    } else if filename.contains("bc2") {
        "BC2".to_string()
    } else if filename.contains("bc3") {
        "BC3".to_string()
    } else if filename.contains("bc7") {
        "BC7".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_format_from_filename() {
        assert_eq!(extract_format_from_filename("r2-256-bc1.dds"), "BC1");
        assert_eq!(extract_format_from_filename("r2-256-bc2.dds"), "BC2");
        assert_eq!(extract_format_from_filename("r2-256-bc3.dds"), "BC3");
        assert_eq!(extract_format_from_filename("r2-256-bc7.dds"), "BC7");
        assert_eq!(extract_format_from_filename("unknown.dds"), "Unknown");
    }

    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir().unwrap();
        assert!(temp_dir.exists());

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}

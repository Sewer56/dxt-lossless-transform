use super::file_compare;
use dxt_lossless_transform_dds::dds::{parse_dds, DdsFormat};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Error types for endianness testing
#[derive(Debug, thiserror::Error)]
pub enum EndianTestError {
    #[error("External tool not available: {0}")]
    ToolNotFound(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("File comparison failed: {expected} != {actual}")]
    #[allow(dead_code)]
    FilesDiffer { expected: String, actual: String },

    #[error("Tests failed: {0}")]
    TestsFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Test result for a single format
#[derive(Debug)]
pub struct EndianTestResult {
    pub format: String,
    pub transform_success: bool,
    pub untransform_success: bool,
    pub files_identical: bool,
}

impl EndianTestResult {
    /// Check if this test result represents a successful test
    pub fn is_success(&self) -> bool {
        self.transform_success && self.untransform_success && self.files_identical
    }
}

/// Cross-compilation targets
const LITTLE_ENDIAN_TARGET: &str = "x86_64-unknown-linux-gnu";
const BIG_ENDIAN_TARGET: &str = "powerpc64-unknown-linux-gnu";

/// Check if a DDS file is a supported format for endian testing
/// Currently supports BC1 and BC2, excludes BC3 and BC7 (not ready yet)
fn is_supported_format(file_path: &Path) -> Result<bool, EndianTestError> {
    let file_data = fs::read(file_path)?;

    match parse_dds(&file_data) {
        Some(dds_info) => {
            match dds_info.format {
                DdsFormat::BC1 | DdsFormat::BC2 => Ok(true),
                DdsFormat::BC3 | DdsFormat::BC7 => Ok(false), // Not ready yet
                _ => Ok(false),                               // Other formats not supported
            }
        }
        None => Ok(false), // Not a valid DDS file
    }
}

/// Helper function to navigate up multiple parent directories
fn go_up_parents(path: &Path, levels: usize) -> &Path {
    let mut current = path;
    for _ in 0..levels {
        current = current.parent().unwrap();
    }
    current
}

/// Get all .dds files from the assets test directory
fn get_test_files() -> Result<Vec<fs::DirEntry>, EndianTestError> {
    use crate::util::find_all_files;

    let workspace_root = go_up_parents(Path::new(env!("CARGO_MANIFEST_DIR")), 3);
    let assets_dir = workspace_root.join("assets").join("tests");
    let mut entries = Vec::new();

    find_all_files(&assets_dir, &mut entries)?;

    let mut dds_files: Vec<fs::DirEntry> = entries
        .into_iter()
        .filter(|entry| {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                let name = file_name.to_string_lossy();
                // First check if it's a .dds file by extension
                if !name.to_lowercase().ends_with(".dds") {
                    return false;
                }

                // Then check if it's a supported format (BC1/BC2) by parsing the file content
                is_supported_format(&path).unwrap_or_default()
            } else {
                false
            }
        })
        .collect();

    // Sort by file name for consistent ordering
    dds_files.sort_by_key(|a| a.file_name());
    Ok(dds_files)
}

/// Run comprehensive endianness tests for all files
pub fn run_all_endian_tests() -> Result<Vec<EndianTestResult>, EndianTestError> {
    let mut results = Vec::new();
    let test_files = get_test_files()?;

    println!("Testing endianness for all .dds files in assets directory");
    println!("Found {} .dds files", test_files.len());

    // Test each .dds file in the assets/tests directory
    for test_file in &test_files {
        let file_name_os = test_file.file_name();
        let file_name = file_name_os.to_string_lossy();
        println!("Processing file: {file_name}");

        let result = run_single_endian_test(test_file)?;

        if result.is_success() {
            println!("  ✓ Success");
        } else {
            println!("  ❌ Failed");
        }

        results.push(result);
    }

    Ok(results)
}

/// Run endianness test for a single file
fn run_single_endian_test(test_file: &fs::DirEntry) -> Result<EndianTestResult, EndianTestError> {
    // Create isolated temporary directories with random names in project-relative location
    let project_root = go_up_parents(Path::new(env!("CARGO_MANIFEST_DIR")), 3);
    let tmp_base = project_root.join("tmp");
    fs::create_dir_all(&tmp_base)?;

    let temp_dir = TempDir::new_in(&tmp_base)?;
    let base_name = file_compare::generate_random_dir_name("endian_test");

    let little_endian_dir = temp_dir.path().join(format!("{base_name}_le"));
    let big_endian_dir = temp_dir.path().join(format!("{base_name}_be"));
    let transform_le_dir = temp_dir.path().join(format!("{base_name}_transform_le"));
    let transform_be_dir = temp_dir.path().join(format!("{base_name}_transform_be"));
    let untransform_le_dir = temp_dir.path().join(format!("{base_name}_untransform_le"));
    let untransform_be_dir = temp_dir.path().join(format!("{base_name}_untransform_be"));

    // Create all directories
    for dir in &[
        &little_endian_dir,
        &big_endian_dir,
        &transform_le_dir,
        &transform_be_dir,
        &untransform_le_dir,
        &untransform_be_dir,
    ] {
        fs::create_dir_all(dir)?;
    }

    // Copy test file to input directories
    let test_file_path = test_file.path();
    let file_name = test_file.file_name();

    fs::copy(&test_file_path, little_endian_dir.join(&file_name))?;
    fs::copy(&test_file_path, big_endian_dir.join(&file_name))?;

    // Test transform operations
    let input_le_file = little_endian_dir.join(&file_name);
    let input_be_file = big_endian_dir.join(&file_name);

    let transform_le_success =
        run_transform_command(LITTLE_ENDIAN_TARGET, &input_le_file, &transform_le_dir)?;

    let transform_be_success =
        run_transform_command(BIG_ENDIAN_TARGET, &input_be_file, &transform_be_dir)?;

    let transform_success = transform_le_success && transform_be_success;

    // Compare transform outputs
    let transform_files_identical = if transform_success {
        file_compare::compare_directories(&transform_le_dir, &transform_be_dir)?
    } else {
        false
    };

    // Test untransform operations for both little and big endian (if transforms succeeded)
    let (untransform_success, untransform_files_identical) = if transform_success {
        // Run untransform for both endianness targets
        let untransform_le_success =
            run_untransform_command(LITTLE_ENDIAN_TARGET, &transform_le_dir, &untransform_le_dir)?;

        let untransform_be_success =
            run_untransform_command(BIG_ENDIAN_TARGET, &transform_be_dir, &untransform_be_dir)?;

        let untransform_success = untransform_le_success && untransform_be_success;

        // Compare untransform directories for endianness consistency
        let untransform_files_identical = if untransform_success {
            file_compare::compare_directories(&untransform_le_dir, &untransform_be_dir)?
        } else {
            false
        };

        (untransform_success, untransform_files_identical)
    } else {
        (false, false)
    };

    Ok(EndianTestResult {
        format: file_name.to_string_lossy().to_string(),
        transform_success,
        untransform_success,
        files_identical: transform_files_identical && untransform_files_identical,
    })
}

/// Run transform command using cross for specified target
fn run_transform_command(
    target: &str,
    input_file: &Path,
    output_file: &Path,
) -> Result<bool, EndianTestError> {
    // Set target-specific cargo target directory to prevent artifact collisions
    let project_root = go_up_parents(Path::new(env!("CARGO_MANIFEST_DIR")), 3);
    let target_dir = project_root.join("target").join(format!("cross-{target}"));

    let output = Command::new("cross")
        .env("CARGO_TARGET_DIR", target_dir)
        .args([
            "run",
            "--release",
            "--target",
            target,
            "--features",
            "debug-endian",
            "--bin",
            "dxt-lossless-transform-cli",
            "--",
            "debug-endian-transform",
            "--input",
            input_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Transform failed for target {target}: {stderr}");
        return Ok(false);
    }

    Ok(true)
}

/// Run untransform command using cross for specified target
fn run_untransform_command(
    target: &str,
    input_file: &Path,
    output_file: &Path,
) -> Result<bool, EndianTestError> {
    // Set target-specific cargo target directory to prevent artifact collisions
    let project_root = go_up_parents(Path::new(env!("CARGO_MANIFEST_DIR")), 3);
    let target_dir = project_root.join("target").join(format!("cross-{target}"));

    let output = Command::new("cross")
        .env("CARGO_TARGET_DIR", target_dir)
        .args([
            "run",
            "--release",
            "--target",
            target,
            "--features",
            "debug-endian",
            "--bin",
            "dxt-lossless-transform-cli",
            "--",
            "debug-endian-untransform",
            "--input",
            input_file.to_str().unwrap(),
            "--output",
            output_file.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Untransform failed for target {target}: {stderr}");
        return Ok(false);
    }

    Ok(true)
}

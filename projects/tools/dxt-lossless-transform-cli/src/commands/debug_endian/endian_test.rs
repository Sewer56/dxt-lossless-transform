use super::file_compare;
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

/// Get all .dds files from the assets test directory
fn get_test_files() -> Result<Vec<fs::DirEntry>, EndianTestError> {
    use crate::util::find_all_files;

    let assets_dir = std::env::current_dir()?.join("assets").join("tests");
    let mut entries = Vec::new();

    find_all_files(&assets_dir, &mut entries)?;

    let mut dds_files: Vec<fs::DirEntry> = entries
        .into_iter()
        .filter(|entry| {
            let path = entry.path();
            if let Some(file_name) = path.file_name() {
                file_name.to_string_lossy().ends_with(".dds")
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
    // Create isolated temporary directories with random names
    let temp_dir = TempDir::new()?;
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
    let output = Command::new("cross")
        .args([
            "run",
            "--target",
            target,
            "--all-features",
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
    let output = Command::new("cross")
        .args([
            "run",
            "--target",
            target,
            "--all-features",
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

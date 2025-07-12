mod endian_test;
mod file_compare;
mod tool_check;

use crate::error::TransformError;
use argh::FromArgs;
use std::time::Instant;

#[derive(FromArgs, Debug)]
/// Debug commands for testing cross-endian DXT transform operations
#[argh(subcommand, name = "debug-endian")]
pub struct DebugEndianCmd {
    /// skip tool availability checks (for testing)
    #[argh(switch)]
    skip_tool_check: bool,
}

/// Test result for a single endianness validation
#[derive(Debug)]
#[allow(dead_code)]
pub struct EndianTestResult {
    pub format: String,
    pub preset: String,
    pub transform_success: bool,
    pub untransform_success: bool,
    pub files_identical: bool,
}

#[allow(dead_code)]
impl EndianTestResult {
    /// Helper to check if this test result represents a complete success
    pub fn is_successful(&self) -> bool {
        self.files_identical && self.transform_success && self.untransform_success
    }
}

pub fn handle_debug_command(cmd: DebugEndianCmd) -> Result<(), TransformError> {
    println!("=== DXT Lossless Transform Endianness Testing ===");
    println!("This command validates that transform operations produce identical results");
    println!("across little-endian and big-endian architectures.\n");

    let start = Instant::now();

    // Check tool availability first (fail fast)
    if !cmd.skip_tool_check {
        println!("Checking external tool availability...");
        tool_check::verify_all_tools()?;
        println!("✓ All required tools are available\n");
    } else {
        println!("⚠ Skipping tool availability checks\n");
    }

    // Run endianness tests for all formats and presets
    println!("Running endianness validation tests...");
    let test_files = [
        "r2-256-bc1.dds",
        "r2-256-bc2.dds",
        "r2-256-bc3.dds",
        "r2-256-bc7.dds",
    ];
    let presets = ["low", "medium", "optimal", "max"];

    let mut results = Vec::new();
    let mut total_tests = 0;
    let mut successful_tests = 0;

    for file in &test_files {
        for preset in &presets {
            total_tests += 1;
            println!("Testing {file} with {preset} preset...");

            match endian_test::run_single_test(file, preset) {
                Ok(result) => {
                    if result.files_identical {
                        successful_tests += 1;
                        println!("  ✓ PASS: Files identical across endianness");
                    } else {
                        println!("  ✗ FAIL: Files differ between little and big endian");
                    }
                    results.push(result);
                }
                Err(e) => {
                    println!("  ✗ ERROR: {e}");
                    // Continue with other tests
                }
            }
        }
    }

    let elapsed = start.elapsed();

    // Report final results
    println!("\n=== Endianness Testing Complete ===");
    println!("Time taken: {elapsed:.2?}");
    println!("Tests passed: {successful_tests}/{total_tests}");

    if successful_tests == total_tests {
        println!("🎉 All endianness tests PASSED! DXT transforms are endian-agnostic.");
    } else {
        println!("⚠ Some endianness tests FAILED! Manual investigation required.");
        return Err(TransformError::Debug(format!(
            "Endianness testing failed: {successful_tests}/{total_tests} tests passed"
        )));
    }

    Ok(())
}

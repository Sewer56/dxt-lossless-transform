use argh::FromArgs;
use std::error::Error;
use std::path::PathBuf;

mod endian_test;
mod file_compare;
mod tool_check;
mod transform_command;

use crate::util::canonicalize_cli_path;
use endian_test::EndianTestError;

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "debug-endian")]
/// Debug endianness compatibility by testing DXT transforms across architectures
pub struct DebugEndianCmd {
    /// skip tool availability checks (for testing)
    #[argh(switch)]
    skip_tool_check: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "debug-endian-transform")]
/// Transform a single file with all manual combinations (for debug-endian testing)
pub struct DebugEndianTransformCmd {
    /// input file path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "debug-endian-untransform")]
/// Untransform all files in input directory (for debug-endian testing)
pub struct DebugEndianUntransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

/// Handle the debug endian transform command
pub fn handle_debug_endian_transform_command(
    cmd: DebugEndianTransformCmd,
) -> Result<(), Box<dyn Error>> {
    transform_command::handle_transform_single_file(cmd.input, cmd.output)
}

/// Handle the debug endian untransform command
pub fn handle_debug_endian_untransform_command(
    cmd: DebugEndianUntransformCmd,
) -> Result<(), Box<dyn Error>> {
    transform_command::handle_untransform_single_file(cmd.input, cmd.output)
}
/// Handle the debug endian command execution
pub fn handle_debug_command(cmd: DebugEndianCmd) -> Result<(), Box<dyn Error>> {
    println!("=== DXT Endianness Testing ===");
    println!("Testing transform operations across little-endian and big-endian architectures");
    println!("This ensures byte-for-byte identical results regardless of target endianness.\n");

    // Check tool availability first (fail fast)
    if !cmd.skip_tool_check {
        println!("Checking external tool availability...");
        tool_check::verify_all_tools()?;
        println!("✓ All required tools available\n");
    } else {
        println!("⚠ Skipping tool availability checks\n");
    }

    // Run endianness tests for all combinations
    println!("Running comprehensive endianness validation...");
    let results = endian_test::run_all_endian_tests()?;

    // Report results
    report_results(&results)?;

    Ok(())
}

/// Report comprehensive test results
fn report_results(results: &[endian_test::EndianTestResult]) -> Result<(), EndianTestError> {
    let total_tests = results.len();
    let successful_tests = results.iter().filter(|r| r.is_success()).count();

    println!("\n=== Endianness Test Results ===");
    println!("Total combinations tested: {total_tests}");
    println!("Successful: {successful_tests}");
    println!("Failed: {}", total_tests - successful_tests);

    if successful_tests == total_tests {
        println!("✅ All endianness tests passed! Transform operations are endian-agnostic.");
    } else {
        println!("❌ Some endianness tests failed!");

        // Report failed tests
        for result in results.iter().filter(|r| !r.is_success()) {
            println!("  Failed: {}", result.format);
            if !result.transform_success {
                println!("    - Transform operation failed");
            }
            if !result.untransform_success {
                println!("    - Untransform operation failed");
            }
            if !result.files_identical {
                println!("    - File outputs differ between endianness");
            }
        }

        return Err(EndianTestError::TestsFailed(format!(
            "{} out of {} tests failed",
            total_tests - successful_tests,
            total_tests
        )));
    }

    Ok(())
}

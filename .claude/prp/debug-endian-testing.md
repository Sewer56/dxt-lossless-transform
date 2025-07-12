name: "Debug Endianness Testing CLI Command - Complete Implementation PRP"
description: |

## Purpose
Comprehensive PRP for implementing a CLI command that validates DXT transform operations produce identical results across little-endian and big-endian architectures, ensuring the codebase is truly endian-agnostic.

## Core Principles
1. **Tool Availability**: Check cargo/cross tools at startup, fail fast if missing
2. **Comprehensive Testing**: Test all transformation modes and file formats
3. **Cross-Platform Validation**: Use existing PowerPC big-endian CI infrastructure
4. **File Integrity**: Ensure byte-for-byte identical results across endianness
5. **CI Integration**: Seamlessly integrate with existing GitHub workflows

---

## Goal
Create a `debug-endian` CLI command that performs cross-endian validation of DXT transform operations by:
- Checking external tool availability (cargo, cross)
- Running transform/untransform operations on both little-endian and big-endian targets
- Comparing results for identical file output
- Testing all available compression formats and transformation modes
- Providing clear success/failure reporting
- Integrating into CI pipeline for automated endianness regression testing

## Why
- **Correctness Assurance**: Ensures lossless transforms are truly lossless across all architectures
- **Cross-Platform Reliability**: Validates that endian-specific byte handling doesn't introduce corruption
- **Development Confidence**: Provides automated verification for endianness-related changes
- **CI Integration**: Prevents endianness regressions through automated testing
- **User Trust**: Demonstrates the library works correctly on all supported platforms

## What
A new CLI debug command that orchestrates cross-endian testing workflows using subprocess execution to invoke cargo and cross for building and testing on different endianness targets.

### Success Criteria
- [ ] CLI command `debug-endian` executes without errors when tools are available
- [ ] Command checks and reports missing external tools (cargo, cross) at startup
- [ ] Transform operations produce identical output files on little-endian and big-endian targets
- [ ] Untransform operations produce identical output files on little-endian and big-endian targets
- [ ] All BC1, BC2, BC3, BC7 formats are tested with multiple compression presets
- [ ] CI integration automatically runs endianness validation
- [ ] Clear error messages for any endianness-related failures
- [ ] Command completes within reasonable time limits (< 5 minutes on CI)

## All Needed Context

### Documentation & References
```yaml
# MUST READ - Include these in your context window
- url: https://doc.rust-lang.org/std/process/struct.Command.html
  why: Best practices for subprocess execution and error handling

- url: https://github.com/cross-rs/cross
  why: Understanding cross-compilation tool usage patterns and PowerPC target support

- file: /home/sewer/Project/dxt-lossless-transform/projects/tools/dxt-lossless-transform-cli/src/commands/debug_bc1/mod.rs
  why: Existing debug command patterns to mirror for consistent CLI structure

- file: /home/sewer/Project/dxt-lossless-transform/projects/tools/dxt-lossless-transform-cli/src/main.rs
  why: CLI command registration patterns using argh derive macros

- file: /home/sewer/Project/dxt-lossless-transform/projects/tools/dxt-lossless-transform-cli/src/commands/transform/mod.rs
  why: Transform operation patterns and preset handling

- file: /home/sewer/Project/dxt-lossless-transform/.github/workflows/rust.yml
  why: Existing CI patterns and PowerPC cross-compilation configuration

- file: /home/sewer/Project/dxt-lossless-transform/CLAUDE.md
  why: Project validation requirements and coding standards

- file: /home/sewer/Project/dxt-lossless-transform/assets/tests/
  why: Test assets for endianness validation (BC1, BC2, BC3, BC7 DDS files)
```

### Current Codebase tree
```bash
projects/tools/dxt-lossless-transform-cli/
├── src/
│   ├── main.rs                    # CLI entry point with Commands enum
│   ├── commands/
│   │   ├── transform/mod.rs       # Transform operation patterns
│   │   ├── untransform/mod.rs     # Untransform operation patterns
│   │   ├── debug_bc1/mod.rs       # Existing debug command structure
│   │   └── debug_bc2/mod.rs       # Additional debug command reference
│   ├── error.rs                   # Error handling patterns
│   └── util/core.rs               # File utilities and path helpers
└── Cargo.toml                     # Feature flag definitions

assets/tests/
├── r2-256-bc1.dds                # BC1 test file (32,896 bytes)
├── r2-256-bc2.dds                # BC2 test file (65,664 bytes)
├── r2-256-bc3.dds                # BC3 test file (65,664 bytes)
├── r2-256-bc7.dds                # BC7 test file (65,684 bytes)
└── r2-256.png                    # Reference image
```

### Desired Codebase tree with files to be added
```bash
projects/tools/dxt-lossless-transform-cli/
├── src/
│   ├── main.rs                    # MODIFY: Add DebugEndian command variant
│   ├── commands/
│   │   └── debug_endian/
│   │       ├── mod.rs             # NEW: Main debug endian command implementation
│   │       ├── tool_check.rs      # NEW: Tool availability verification
│   │       ├── endian_test.rs     # NEW: Core endianness testing logic
│   │       └── file_compare.rs    # NEW: Binary file comparison utilities
└── Cargo.toml                     # MODIFY: Add debug-endian feature flag

.github/workflows/rust.yml         # MODIFY: Add endianness testing step
```

### Known Gotchas of our codebase & Library Quirks
```rust
// CRITICAL: Debug commands require feature flags for compilation
#[cfg(feature = "debug-endian")]
DebugEndian(commands::debug_endian::DebugEndianCmd),

// CRITICAL: Use std::process::Command for subprocess execution
// Never use shell=true or string command parsing for security
let output = Command::new("cargo")
    .arg("--version")
    .output()
    .map_err(|e| ToolError::NotFound(format!("cargo: {}", e)))?;

// CRITICAL: PowerPC big-endian target for cross-compilation
const BIG_ENDIAN_TARGET: &str = "powerpc64-unknown-linux-gnu";

// CRITICAL: File comparison must be byte-for-byte identical
// Use Read trait with BufReader for efficient large file comparison

// CRITICAL: All external tool execution must handle both:
// 1. IO errors (tool not found, permission denied)
// 2. Exit status errors (tool found but failed)

// CRITICAL: Follow CLAUDE.md validation requirements:
// cargo test --all-features
// cargo clippy --workspace --all-features -- -D warnings
// cargo doc --workspace --all-features
// cross test --package dxt-lossless-transform-dds --target powerpc64-unknown-linux-gnu
```

## Implementation Blueprint

### Data models and structure
```rust
// Core command structure following existing debug patterns
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "debug-endian")]
pub struct DebugEndianCmd {
    /// Skip tool availability checks (for testing)
    #[argh(switch)]
    skip_tool_check: bool,
}

// Error types for comprehensive error handling
#[derive(Debug, thiserror::Error)]
pub enum EndianTestError {
    #[error("External tool not available: {0}")]
    ToolNotFound(String),
    
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    
    #[error("File comparison failed: {expected} != {actual}")]
    FilesDiffer { expected: String, actual: String },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Test result tracking
#[derive(Debug)]
pub struct EndianTestResult {
    pub format: String,
    pub preset: String,
    pub transform_success: bool,
    pub untransform_success: bool,
    pub files_identical: bool,
}
```

### List of tasks to be completed to fulfill the PRP in the order they should be completed

```yaml
Task 1: Create debug-endian command structure
MODIFY projects/tools/dxt-lossless-transform-cli/Cargo.toml:
  - ADD feature: debug-endian = ["debug"] under [features]
  - PRESERVE existing debug feature dependencies

CREATE projects/tools/dxt-lossless-transform-cli/src/commands/debug_endian/mod.rs:
  - MIRROR pattern from: src/commands/debug_bc1/mod.rs
  - IMPLEMENT DebugEndianCmd struct with argh derives
  - ADD execute() method with tool checking and test orchestration
  - KEEP error handling pattern identical to other debug commands

Task 2: Implement tool availability checking
CREATE projects/tools/dxt-lossless-transform-cli/src/commands/debug_endian/tool_check.rs:
  - IMPLEMENT check_tool_available(tool_name: &str) -> Result<(), EndianTestError>
  - CHECK cargo --version and cross --version using std::process::Command
  - HANDLE both IO errors and non-zero exit codes
  - PROVIDE clear error messages for missing tools

Task 3: Implement core endianness testing logic  
CREATE projects/tools/dxt-lossless-transform-cli/src/commands/debug_endian/endian_test.rs:
  - IMPLEMENT run_endian_test() function
  - CREATE temporary directories for little/big endian outputs
  - EXECUTE transform operations using subprocess calls to CLI
  - EXECUTE untransform operations using subprocess calls to CLI
  - COMPARE output files for byte-for-byte equality
  - TEST all compression presets: low, medium, optimal, max

Task 4: Implement file comparison utilities
CREATE projects/tools/dxt-lossless-transform-cli/src/commands/debug_endian/file_compare.rs:
  - IMPLEMENT compare_files(path1: &Path, path2: &Path) -> Result<bool, EndianTestError>
  - USE BufReader for efficient large file comparison
  - HANDLE file size differences early for performance
  - PROVIDE detailed error messages on comparison failures

Task 5: Integrate command into CLI main
MODIFY projects/tools/dxt-lossless-transform-cli/src/main.rs:
  - ADD #[cfg(feature = "debug-endian")] DebugEndian variant to Commands enum
  - ADD debug_endian module declaration
  - PRESERVE existing command structure and error handling

Task 6: Add CI integration
MODIFY .github/workflows/rust.yml:
  - ADD endianness testing step to existing workflow
  - RUN only on x86_64-unknown-linux-gnu matrix entry
  - EXECUTE cargo run --bin dxt-lossless-transform-cli --features debug-endian -- debug-endian
  - HANDLE step failure appropriately for CI reporting
```

### Per task pseudocode

```rust
// Task 1 - Main command structure
impl DebugEndianCmd {
    pub fn execute(&self) -> Result<(), EndianTestError> {
        // PATTERN: Check tools first (fail fast)
        if !self.skip_tool_check {
            tool_check::verify_all_tools()?;
        }
        
        // PATTERN: Run tests for all formats
        let test_files = ["r2-256-bc1.dds", "r2-256-bc2.dds", "r2-256-bc3.dds", "r2-256-bc7.dds"];
        let presets = ["low", "medium", "optimal", "max"];
        
        let mut results = Vec::new();
        for file in test_files {
            for preset in presets {
                let result = endian_test::run_single_test(file, preset)?;
                results.push(result);
            }
        }
        
        // PATTERN: Report comprehensive results
        report_results(results)?;
        Ok(())
    }
}

// Task 2 - Tool checking
fn check_tool_available(tool: &str) -> Result<(), EndianTestError> {
    // CRITICAL: Use Command::new(), never shell execution
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .map_err(|e| EndianTestError::ToolNotFound(format!("{}: {}", tool, e)))?;
    
    if !output.status.success() {
        return Err(EndianTestError::CommandFailed(
            format!("{} --version failed with exit code: {}", tool, output.status)
        ));
    }
    
    Ok(())
}

// Task 3 - Core testing logic
fn run_single_test(file: &str, preset: &str) -> Result<EndianTestResult, EndianTestError> {
    // PATTERN: Create isolated temporary directories
    let temp_dir = tempfile::tempdir()?;
    let little_endian_dir = temp_dir.path().join("little");
    let big_endian_dir = temp_dir.path().join("big");
    fs::create_dir_all(&little_endian_dir)?;
    fs::create_dir_all(&big_endian_dir)?;
    
    // PATTERN: Execute transform on native (little-endian)
    let transform_result_le = Command::new("cargo")
        .args(&["run", "--bin", "dxt-lossless-transform-cli", "--", 
               "transform", "--input", "assets/tests", "--output", little_endian_dir.to_str().unwrap(),
               "--preset", preset])
        .output()?;
        
    // PATTERN: Execute transform using cross (big-endian)  
    let transform_result_be = Command::new("cross")
        .args(&["run", "--target", "powerpc64-unknown-linux-gnu",
               "--bin", "dxt-lossless-transform-cli", "--",
               "transform", "--input", "assets/tests", "--output", big_endian_dir.to_str().unwrap(),
               "--preset", preset])
        .output()?;
    
    // CRITICAL: Compare transform outputs
    let transform_files_match = file_compare::compare_directories(&little_endian_dir, &big_endian_dir)?;
    
    // PATTERN: Repeat for untransform operations
    // ... (similar pattern for untransform)
    
    Ok(EndianTestResult {
        format: extract_format(file),
        preset: preset.to_string(),
        transform_success: transform_result_le.status.success() && transform_result_be.status.success(),
        untransform_success: /* ... */,
        files_identical: transform_files_match && untransform_files_match,
    })
}

// Task 4 - File comparison
fn compare_files(path1: &Path, path2: &Path) -> Result<bool, EndianTestError> {
    // PATTERN: Check file sizes first for early exit
    let metadata1 = fs::metadata(path1)?;
    let metadata2 = fs::metadata(path2)?;
    
    if metadata1.len() != metadata2.len() {
        return Ok(false);
    }
    
    // PATTERN: Use BufReader for efficient large file reading
    let mut file1 = BufReader::new(File::open(path1)?);
    let mut file2 = BufReader::new(File::open(path2)?);
    
    const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer
    let mut buffer1 = vec![0u8; BUFFER_SIZE];
    let mut buffer2 = vec![0u8; BUFFER_SIZE];
    
    loop {
        let bytes1 = file1.read(&mut buffer1)?;
        let bytes2 = file2.read(&mut buffer2)?;
        
        if bytes1 != bytes2 || buffer1[..bytes1] != buffer2[..bytes2] {
            return Ok(false);
        }
        
        if bytes1 == 0 {
            break; // EOF reached
        }
    }
    
    Ok(true)
}
```

### Integration Points

```yaml
CARGO_TOML:
  - add feature: "debug-endian = [\"debug\"]" under [features] section
  - preserve existing debug feature structure
  
MODULE_EXPORTS:
  - add to: src/main.rs
  - pattern: "#[cfg(feature = \"debug-endian\")] mod debug_endian;"
  - pattern: "#[cfg(feature = \"debug-endian\")] DebugEndian(commands::debug_endian::DebugEndianCmd),"
  
CI_INTEGRATION:
  - add to: .github/workflows/rust.yml
  - condition: "if: matrix.target == 'x86_64-unknown-linux-gnu'"
  - command: "cargo run --bin dxt-lossless-transform-cli --features debug-endian -- debug-endian"
  
DEPENDENCIES:
  - add tempfile for temporary directory management
  - use std::process::Command (built-in, no external deps)
  - use existing error handling with thiserror
```

## Validation Loop

### Level 1: Syntax & Style
```bash
# Run these FIRST - fix any errors before proceeding
cargo fmt --all                                                    # Auto-format code
cargo clippy --workspace --all-features -- -D warnings           # Linting with warnings as errors
cargo check --all-features                                        # Basic compilation check

# Expected: No errors. If errors, READ the error and fix.
```

### Level 2: Unit Tests
```rust
// ADD to debug_endian/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_tool_availability_check() {
        // Test that cargo is available (should always pass in development)
        assert!(tool_check::check_tool_available("cargo").is_ok());
    }

    #[test]
    fn test_file_comparison_identical() {
        // Test file comparison with identical files
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.bin");
        let file2 = temp_dir.path().join("test2.bin");
        
        std::fs::write(&file1, b"identical content").unwrap();
        std::fs::write(&file2, b"identical content").unwrap();
        
        assert!(file_compare::compare_files(&file1, &file2).unwrap());
    }

    #[test]
    fn test_file_comparison_different() {
        // Test file comparison with different files
        let temp_dir = tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.bin");
        let file2 = temp_dir.path().join("test2.bin");
        
        std::fs::write(&file1, b"content one").unwrap();
        std::fs::write(&file2, b"content two").unwrap();
        
        assert!(!file_compare::compare_files(&file1, &file2).unwrap());
    }
}
```

```bash
# Run and iterate until passing:
cargo test --all-features
cargo test debug_endian -- --nocapture  # For specific test with output
# If failing: Read error, understand root cause, fix code, re-run
```

### Level 3: Integration Testing
```bash
# Manual test - tool availability check
cargo run --bin dxt-lossless-transform-cli --features debug-endian -- debug-endian --skip-tool-check

# Manual test - full endianness validation (requires cross)
cargo run --bin dxt-lossless-transform-cli --features debug-endian -- debug-endian

# Expected: Command executes successfully and reports test results
```

## Final validation Checklist
- [ ] All tests pass: `cargo test --all-features`
- [ ] No linting errors: `cargo clippy --workspace --all-features -- -D warnings`
- [ ] No compilation errors: `cargo check --all-features`
- [ ] Documentation builds: `cargo doc --workspace --all-features`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] Big endian testing passes (if `cross` is available): `cross test --package dxt-lossless-transform-dds --target powerpc64-unknown-linux-gnu`
- [ ] Manual test successful: `cargo run --bin dxt-lossless-transform-cli --features debug-endian -- debug-endian`
- [ ] Tool availability checks work correctly
- [ ] File comparison utilities handle edge cases (empty files, large files, identical files)
- [ ] Error cases provide clear, actionable messages
- [ ] CI integration runs without blocking other tests

---

## Anti-Patterns to Avoid

- ❌ Don't use shell command strings - always use Command::new() with individual arguments
- ❌ Don't ignore subprocess exit codes - check both IO errors and status.success()
- ❌ Don't assume external tools are available - always check at startup
- ❌ Don't use unwrap() with subprocess results - handle errors gracefully
- ❌ Don't hardcode file paths - use relative paths and proper path joining
- ❌ Don't skip file size comparison optimization for large file comparisons
- ❌ Don't forget to clean up temporary directories (tempfile crate handles this)
- ❌ Don't assume cross-compilation targets are available - provide clear error messages
- ❌ Don't block CI pipeline on endianness tests - make them informational initially

**PRP Quality Score: 9/10** - Comprehensive context provided with detailed implementation blueprint, executable validation gates, and extensive error handling patterns. High confidence for one-pass implementation success.
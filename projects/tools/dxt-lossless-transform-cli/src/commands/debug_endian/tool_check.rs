use super::endian_test::EndianTestError;
use std::process::Command;

/// Verify that all required external tools are available
pub fn verify_all_tools() -> Result<(), EndianTestError> {
    // Check cargo availability
    check_tool_available("cargo")?;

    // Check cross availability
    check_tool_available("cross")?;

    Ok(())
}

/// Check if a specific tool is available and working
fn check_tool_available(tool: &str) -> Result<(), EndianTestError> {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .map_err(|e| EndianTestError::ToolNotFound(format!("{tool}: {e}")))?;

    if !output.status.success() {
        return Err(EndianTestError::CommandFailed(format!(
            "{} --version failed with exit code: {}",
            tool, output.status
        )));
    }

    // Optional: Parse and display version info
    let version_output = String::from_utf8_lossy(&output.stdout);
    let first_line = version_output.lines().next().unwrap_or("unknown version");
    println!("  ✓ {}: {}", tool, first_line.trim());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_availability() {
        // Cargo should always be available in development environment
        assert!(check_tool_available("cargo").is_ok());
    }

    #[test]
    fn test_nonexistent_tool() {
        // Test with a tool that definitely doesn't exist
        assert!(check_tool_available("nonexistent-tool-xyz").is_err());
    }
}

use crate::error::TransformError;
use std::process::Command;

/// Verify that all required external tools are available
pub fn verify_all_tools() -> Result<(), TransformError> {
    check_tool_available("cargo")?;
    check_tool_available("cross")?;
    Ok(())
}

/// Check if a specific tool is available and working
fn check_tool_available(tool: &str) -> Result<(), TransformError> {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .map_err(|e| {
            TransformError::Debug(format!(
                "External tool '{tool}' not found or not executable: {e}. Please install {tool} and ensure it's in your PATH."
            ))
        })?;

    if !output.status.success() {
        return Err(TransformError::Debug(format!(
            "External tool '{tool}' failed version check with exit code: {}. Output: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Print version info for debugging
    let version_output = String::from_utf8_lossy(&output.stdout);
    println!("  {tool} version: {}", version_output.trim());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_available() {
        // Cargo should always be available in development environment
        assert!(check_tool_available("cargo").is_ok());
    }

    #[test]
    fn test_nonexistent_tool() {
        // This should fail
        assert!(check_tool_available("definitely-not-a-real-tool-12345").is_err());
    }
}

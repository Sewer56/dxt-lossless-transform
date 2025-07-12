use crate::error::TransformError;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::Path;

/// Compare two files for byte-for-byte equality
pub fn compare_files(path1: &Path, path2: &Path) -> Result<bool, TransformError> {
    // Check file sizes first for early exit
    let metadata1 = fs::metadata(path1).map_err(|e| {
        TransformError::Debug(format!("Cannot read metadata for {}: {e}", path1.display()))
    })?;
    let metadata2 = fs::metadata(path2).map_err(|e| {
        TransformError::Debug(format!("Cannot read metadata for {}: {e}", path2.display()))
    })?;

    if metadata1.len() != metadata2.len() {
        return Ok(false);
    }

    // If both files are empty, they're identical
    if metadata1.len() == 0 {
        return Ok(true);
    }

    // Use BufReader for efficient large file reading
    let file1 = File::open(path1)
        .map_err(|e| TransformError::Debug(format!("Cannot open file {}: {e}", path1.display())))?;
    let file2 = File::open(path2)
        .map_err(|e| TransformError::Debug(format!("Cannot open file {}: {e}", path2.display())))?;

    let mut reader1 = BufReader::new(file1);
    let mut reader2 = BufReader::new(file2);

    const BUFFER_SIZE: usize = 64 * 1024; // 64KB buffer for efficient reading
    let mut buffer1 = vec![0u8; BUFFER_SIZE];
    let mut buffer2 = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes1 = reader1.read(&mut buffer1).map_err(|e| {
            TransformError::Debug(format!("Error reading from {}: {e}", path1.display()))
        })?;
        let bytes2 = reader2.read(&mut buffer2).map_err(|e| {
            TransformError::Debug(format!("Error reading from {}: {e}", path2.display()))
        })?;

        // Check if we read different amounts of data
        if bytes1 != bytes2 {
            return Ok(false);
        }

        // Check if the content differs
        if buffer1[..bytes1] != buffer2[..bytes2] {
            return Ok(false);
        }

        // If we've reached EOF (bytes read = 0), files are identical
        if bytes1 == 0 {
            break;
        }
    }

    Ok(true)
}

/// Compare all files in two directories recursively
pub fn compare_directories(dir1: &Path, dir2: &Path) -> Result<bool, TransformError> {
    let entries1 = collect_all_files(dir1)?;
    let entries2 = collect_all_files(dir2)?;

    // Check if both directories have the same number of files
    if entries1.len() != entries2.len() {
        return Ok(false);
    }

    // Compare each file pair
    for (rel_path, abs_path1) in &entries1 {
        if let Some((_, abs_path2)) = entries2.iter().find(|(rel, _)| rel == rel_path) {
            if !compare_files(abs_path1, abs_path2)? {
                return Ok(false);
            }
        } else {
            // File exists in dir1 but not in dir2
            return Ok(false);
        }
    }

    Ok(true)
}

/// Collect all files in a directory recursively, returning (relative_path, absolute_path) pairs
fn collect_all_files(dir: &Path) -> Result<Vec<(String, std::path::PathBuf)>, TransformError> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    Ok(files)
}

/// Helper function to collect files recursively
fn collect_files_recursive(
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<(String, std::path::PathBuf)>,
) -> Result<(), TransformError> {
    let entries = fs::read_dir(current_dir).map_err(|e| {
        TransformError::Debug(format!(
            "Cannot read directory {}: {e}",
            current_dir.display()
        ))
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|e| TransformError::Debug(format!("Error reading directory entry: {e}")))?;
        let path = entry.path();

        if path.is_file() {
            let relative = path.strip_prefix(base_dir).map_err(|e| {
                TransformError::Debug(format!("Error computing relative path: {e}"))
            })?;
            files.push((relative.to_string_lossy().to_string(), path));
        } else if path.is_dir() {
            collect_files_recursive(base_dir, &path, files)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_identical_files() {
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test1.bin");
        let file2 = temp_dir.join("test2.bin");

        let content = b"Hello, world! This is test content for file comparison.";
        fs::write(&file1, content).unwrap();
        fs::write(&file2, content).unwrap();

        assert!(compare_files(&file1, &file2).unwrap());

        // Clean up
        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_different_files() {
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("test1.bin");
        let file2 = temp_dir.join("test2.bin");

        fs::write(&file1, b"content one").unwrap();
        fs::write(&file2, b"content two").unwrap();

        assert!(!compare_files(&file1, &file2).unwrap());

        // Clean up
        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }

    #[test]
    fn test_empty_files() {
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("empty1.bin");
        let file2 = temp_dir.join("empty2.bin");

        fs::write(&file1, b"").unwrap();
        fs::write(&file2, b"").unwrap();

        assert!(compare_files(&file1, &file2).unwrap());

        // Clean up
        let _ = fs::remove_file(&file1);
        let _ = fs::remove_file(&file2);
    }
}
